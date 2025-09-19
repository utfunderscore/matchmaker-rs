use crate::entry::{Entry, EntryId};
use crate::gamefinder::Game;
use crate::matchmaker;
use crate::matchmaker::MatchmakerResult;
use crate::queue::{Queue, QueueResult};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{Mutex, MutexGuard};
use tracing::{info, warn};

pub struct QueueTracker {
    pub queues: HashMap<String, Arc<Mutex<Queue>>>,
    pub senders: HashMap<EntryId, Sender<Result<QueueResult, String>>>,
    pub locked: bool,
}

impl QueueTracker {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            senders: HashMap::new(),
            locked: false,
        }
    }

    pub async fn from_file() -> Arc<Mutex<Self>> {
        let tracker = Arc::new(Mutex::new(Self::new()));

        let data = tokio::fs::read_to_string("queues.json").await;
        let Ok(data) = data else {
            info!("No queues.json file found, starting with empty QueueTracker");
            tokio::fs::write("queues.json", "[]").await.ok();
            return tracker;
        };
        let Ok(queues_json) = serde_json::from_str::<Vec<Value>>(&data) else {
            warn!("Failed to parse queues.json, starting with empty QueueTracker");
            return tracker;
        };

        for value in queues_json {
            let Some(name) = value
                .get("name")
                .and_then(|v| v.as_str())
                .map(|x| String::from(x))
            else {
                warn!("Queue in queues.json has no name, skipping");
                continue;
            };
            let Some(matchmaker_id) = value
                .get("matchmaker")
                .and_then(|v| v.as_str())
                .map(|x| String::from(x))
            else {
                warn!("Queue {} in queues.json has no matchmaker, skipping", name);
                continue;
            };
            let Some(settings) = value.get("settings") else {
                warn!("Queue {} in queues.json has no settings, skipping", name);
                continue;
            };

            if Self::create(
                tracker.clone(),
                name.clone(),
                matchmaker_id,
                settings.clone(),
                false,
            )
            .await
            .is_ok()
            {
                info!("Loaded queue {} from file", name);
            } else {
                warn!("Failed to create queue {} from file", name);
            }
        }
        tracker
    }

    pub async fn save_to_file(&self) {
        let mut queues: Vec<Value> = Vec::new();

        for (name, queue_mutex) in &self.queues {
            let queue = queue_mutex.lock().await;
            let matchmaker = queue.matchmaker();
            let matchmaker_type = matchmaker.get_type_name();
            let Ok(settings) = matchmaker.serialize() else {
                warn!("Failed to serialize matchmaker for queue {}", name);
                continue;
            };

            let queue_json = serde_json::json!({
                "name": name,
                "matchmaker": matchmaker_type,
                "settings": settings,
            });
            queues.push(queue_json);
        }

        if let Err(e) = std::fs::write(
            "queues.json",
            serde_json::to_string_pretty(&queues).unwrap(),
        ) {
            warn!("Failed to write queues to file: {}", e);
        }
    }

    pub async fn lock(&mut self) {
        self.locked = true;
    }

    pub async fn create(
        tracker: Arc<Mutex<Self>>,
        name: String,
        matchmaker_id: String,
        settings: Value,
        save: bool,
    ) -> Result<(), Box<dyn Error>> {
        let tracker_copy = tracker.clone();
        let mut tracker_guard = tracker_copy.lock().await;

        if tracker_guard.get_queue(&name).await.is_some() {
            return Err(String::from("Queue already exists").into());
        }

        let matchmaker = matchmaker::deserialize(matchmaker_id, settings)?;

        let queue = Queue::new(name, matchmaker, HashMap::new());
        let queue_id = queue.id.clone();
        let queue_ref = Arc::new(Mutex::new(queue));

        tracker_guard
            .queues
            .insert(queue_id.clone(), queue_ref.clone());

        Self::start_task(tracker, queue_id);

        if save {
            tracker_guard.save_to_file().await;
        }

        Ok(())
    }

    pub async fn join(
        &mut self,
        queue_id: &str,
        entry: Entry,
    ) -> Result<Receiver<Result<QueueResult, String>>, Box<dyn Error>> {
        let (channel_tx, channel_rx): (
            Sender<Result<QueueResult, String>>,
            Receiver<Result<QueueResult, String>>,
        ) = tokio::sync::oneshot::channel::<Result<QueueResult, String>>();

        if self.locked {
            return Err("QueueTracker is locked, no new entries can be added".into());
        }

        let queue = self.queues.get_mut(queue_id).ok_or("Queue not found")?;

        let mut queue = queue.lock().await;

        for entry_player in &entry.players {
            if queue.has_player(entry_player) {
                return Err(format!("Player {} is already in this queue", entry_player).into());
            }
        }

        queue.add_entry(entry.clone())?;
        self.senders.insert(entry.id, channel_tx);

        Ok(channel_rx)
    }
    pub fn get_queues(&self) -> &HashMap<String, Arc<Mutex<Queue>>> {
        &self.queues
    }

    pub async fn get_queue(&self, name: &str) -> Option<Arc<Mutex<Queue>>> {
        self.queues.get(name).map(|x| x.clone())
    }

    pub async fn all_queues_empty(&self) -> bool {
        for queue in self.queues.values() {
            let queue = queue.lock().await;
            if !queue.entries().is_empty() {
                return false;
            }
        }
        true
    }

    fn start_task(tracker: Arc<Mutex<Self>>, queue_id: String) {
        // Start a background task to process queues

        tokio::spawn(async move {
            loop {
                Self::tick_task(tracker.clone(), queue_id.clone()).await;
            }
        });
    }

    pub async fn tick_task(tracker: Arc<Mutex<Self>>, queue_id: String) {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let mut tracker = tracker.lock().await;
        let queue = tracker.get_queue(&queue_id).await;
        let Some(queue) = queue else {
            return;
        };

        let mut queue: MutexGuard<Queue> = queue.lock().await;
        let result = queue.tick();

        match result {
            MatchmakerResult::Matched(teams) => {
                let senders = teams
                    .iter()
                    .flatten()
                    .filter_map(|id| tracker.senders.remove(id))
                    .collect::<Vec<Sender<Result<QueueResult, String>>>>();

                let teams_entries: Vec<Vec<Entry>> = teams
                    .into_iter()
                    .map(|team| {
                        team.iter()
                            .filter_map(|id| queue.remove_entry(id))
                            .collect()
                    })
                    .collect();

                for sender in senders {
                    let game = Game::demo();
                    let _ = sender.send(Ok(QueueResult::new(teams_entries.clone(), game)));
                }
            }
            MatchmakerResult::Error(err, affected) => {}
            MatchmakerResult::Skip(_) => {}
        }
    }
}
