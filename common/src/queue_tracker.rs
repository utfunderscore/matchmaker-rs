use crate::gamefinder::GameFinder;
use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker, MatchmakerResult};
use crate::queue::{Entry, Queue, QueueResult};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::{Duration, sleep};
use tracing::info;
use uuid::Uuid;

pub struct QueueTracker {
    directory: PathBuf,
    queues: HashMap<String, Arc<Mutex<Queue>>>,
    game_finder: Arc<Mutex<GameFinder>>,
}

impl QueueTracker {
    pub fn new<T>(path: T, game_finder: Arc<Mutex<GameFinder>>) -> Result<Self, Box<dyn Error>>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create directory for queue tracker");
        }

        info!("Loading queues...");

        let mut queues: HashMap<String, Arc<Mutex<Queue>>> = HashMap::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let queue_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid file name")?
                    .to_string();

                let queue = Queue::from(path)?;
                queues.insert(queue_name, Arc::new(Mutex::new(queue)));
            }
        }

        let mut tracker = QueueTracker {
            directory: path,
            queues: HashMap::new(),
            game_finder,
        };

        info!("Registering queues...");

        for (name, queue) in queues {
            tracker.register_queue(name, queue)?;
        }

        info!("Queue tracker initialized.");

        Ok(tracker)
    }

    pub fn create_queue(
        &mut self,
        name: String,
        matchmaker: String,
        settings: Value,
    ) -> Result<(), Box<dyn Error>> {
        let deserializer: &Deserializer = matchmaker::get_deserializer(&matchmaker)
            .ok_or(format!("Unknown matchmaker: {matchmaker}"))?;
        let matchmaker: Box<dyn Matchmaker + Send + Sync> = deserializer(settings)?;
        let queue = Queue::new(matchmaker);

        queue.save(&name, &self.directory)?;

        let queue_mutex = Arc::new(Mutex::new(queue));

        self.register_queue(name, queue_mutex)?;

        Ok(())
    }

    pub fn register_queue(
        &mut self,
        name: String,
        queue: Arc<Mutex<Queue>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.queues.contains_key(&name) {
            return Err(format!("Queue '{name}' already exists").into());
        }
        let task_queue = queue.clone();
        let name_clone = name.clone();
        let game_finder = self.game_finder.clone();
        tokio::spawn(async move {
            let name = name_clone;

            loop {
                // Sleep for a short duration to avoid busy waiting
                sleep(Duration::from_millis(100)).await;

                let mut queue: MutexGuard<Queue> = task_queue.lock().await;
                let result = queue.tick().await;

                match result {
                    MatchmakerResult::Matched(team_ids) => {
                        let teams: Result<Vec<Vec<Entry>>, String> =
                            QueueTracker::remove_all(&mut queue, team_ids.clone());
                        let senders: Vec<Sender<QueueResult>> = team_ids
                            .iter()
                            .flat_map(|team| team.iter())
                            .filter_map(|entry| queue.remove_sender(entry))
                            .collect();

                        if let Ok(teams) = teams {
                            let players: Vec<Vec<Uuid>> = teams
                                .iter()
                                .map(|team| team.iter().map(|entry| entry.id()).collect())
                                .collect();

                            let game_finder = game_finder.lock().await;
                            let game = game_finder.find_game(&name, &players).await;
                            drop(game_finder);

                            if let Ok(game) = game {
                                for sender in senders {
                                    let queue_result =
                                        QueueResult::Success(teams.clone(), game.clone());
                                    let _ = sender.send(queue_result);
                                }
                            } else {
                                info!("No game found for queue '{}'", name);
                            }
                        } else {
                            // Handle error in removing entries
                            info!("Error removing entries from queue: {:?}", teams.err());
                        }
                    }
                    MatchmakerResult::Skip(_) => {}
                    MatchmakerResult::Error(err) => {
                        let entries = queue.remove_all();

                        for x in entries {
                            let sender = queue.remove_sender(&x.id());
                            if let Some(sender) = sender {
                                let _ = sender.send(QueueResult::Error(err.clone()));
                            }
                        }
                    }
                }
            }
        });

        self.queues.insert(name, queue);
        Ok(())
    }

    pub fn remove_all(
        queue: &mut MutexGuard<Queue>,
        teams: Vec<Vec<Uuid>>,
    ) -> Result<Vec<Vec<Entry>>, String> {
        let mut results = Vec::new();
        for team in teams {
            let mut entries = Vec::new();
            for id in team {
                let entry = queue.remove_entry(&id).map_err(|e| e.to_string())?;
                entries.push(entry);
            }
            results.push(entries);
        }
        Ok(results)
    }

    pub fn get_queue(&self, name: &str) -> Option<Arc<Mutex<Queue>>> {
        self.queues.get(name).cloned()
    }

    pub fn get_queues(&self) -> &HashMap<String, Arc<Mutex<Queue>>> {
        &self.queues
    }

    pub async fn get_queue_by_player(&self, player_id: &Uuid) -> Option<String> {
        for (name, queue) in &self.queues {
            let queue_guard = queue.lock().await;
            if queue_guard.contains_player(player_id) {
                return Some(name.clone());
            }
        }
        None
    }

    pub async fn all_queues_empty(&self) -> bool {
        for queue in self.queues.values() {
            let queue = queue.lock().await;
            if !queue.get_entries().is_empty() {
                return false;
            }
        }
        true
    }
}
