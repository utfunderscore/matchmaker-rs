use crate::game::providers::{FakeGame, Game, GameProvider};
use crate::matchmaker::implementations::Matchmaker;
use crate::matchmaker::serializer::Registry;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use log::{debug, info};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use uuid::Uuid;

pub struct QueueTicker {
    queue: Queue,
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
    game_producer: Box<dyn GameProvider + Send + Sync>,
    entry_channels: HashMap<Uuid, Sender<Result<Game, String>>>,
    sessions: HashMap<Uuid, Vec<Uuid>>,
}
impl QueueTicker {
    pub fn new(
        queue: Queue,
        matchmaker: Box<dyn Matchmaker + Send + Sync>,
        game_producer: Box<dyn GameProvider + Send + Sync>,
    ) -> Arc<Mutex<Self>> {
        let ticker = Self {
            queue,
            matchmaker,
            game_producer,
            entry_channels: HashMap::new(),
            sessions: HashMap::new(),
        };

        let ticker_arc = Arc::new(Mutex::new(ticker));
        let ticker_ref = Arc::downgrade(&ticker_arc);

        tokio::spawn(async move {
            loop {
                let ticker = ticker_ref.upgrade();

                match ticker {
                    None => break,
                    Some(ticker) => ticker.lock().unwrap().tick(),
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        ticker_arc
    }

    fn tick(&mut self) {
        let matchmaker_result = self.matchmaker.matchmake(&self.queue.entries);
        if matchmaker_result.is_err() {
            debug!(
                "Failed to tick queue {}: {}",
                self.queue.name,
                matchmaker_result.err().unwrap()
            );
            return;
        }
        let teams = matchmaker_result.unwrap();

        println!("teams {teams:?}",);

        let game_result = self.game_producer.get_game_server();
        if game_result.is_err() {
            return;
        }
        let game = game_result.unwrap();

        info!("  - Matchmaker success!");

        for team in teams {
            for entry_id in team {
                if let Err(e) = self.queue.remove_team(&entry_id) {
                    eprintln!("Failed to remove team from the queue: {e}");
                } else {
                    self.try_notify_socket(&entry_id, game.clone());
                }
            }
        }
    }

    fn try_notify_socket(&mut self, entry_id: &Uuid, game: Game) {
        let sender = self.entry_channels.get(entry_id);
        if sender.is_none() {
            return;
        }

        let removed = self.entry_channels.remove(entry_id).unwrap();
        removed.send(Ok(game)).unwrap();
    }

    pub fn add_team(
        &mut self,
        session_id: Uuid,
        queue_entry: QueueEntry,
    ) -> Result<Receiver<Result<Game, String>>, String> {
        let queue_entry_id = queue_entry.id;

        self.matchmaker.validate_entry(&queue_entry)?;
        self.get_queue_mut().add_team(queue_entry)?;

        let (sender, receiver) = channel::<Result<Game, String>>();

        self.add_channel(queue_entry_id, sender);
        self.sessions
            .entry(session_id)
            .or_insert_with(|| vec![queue_entry_id]);

        Ok(receiver)
    }

    pub fn remove_team(&mut self, team_id: &Uuid) -> Result<bool, String> {
        self.entry_channels.remove(team_id);
        for x in self.sessions.values_mut() {
            x.retain_mut(|x| x != team_id);
        }
        self.queue.remove_team(team_id)
    }

    pub fn remove_session(&mut self, session_id: &Uuid) {
        let entry_ids_opt = self.sessions.remove(session_id);
        if let Some(entry_ids) = entry_ids_opt {
            for x in entry_ids {
                let _ = self.remove_team(&x);
            }
        }
    }

    pub fn add_channel(&mut self, entry_id: Uuid, sender: Sender<Result<Game, String>>) {
        self.entry_channels.insert(entry_id, sender);
    }

    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }

    pub fn get_queue_mut(&mut self) -> &mut Queue {
        &mut self.queue
    }
}

// Serialization & Deserialization
impl QueueTicker {
    pub fn save(&self, matchmaker_registry: &Registry) -> Result<Value, String> {
        let mut json: HashMap<String, Value> = HashMap::new();

        let namespace = self.matchmaker.namespace();

        let serializer = matchmaker_registry
            .get(namespace)
            .ok_or("Could not find serializer")?;

        json.insert(String::from("name"), Value::String(self.queue.name.clone()));
        // TODO: Consider serializing players in queue and re-establishing active connections
        // json.insert(String::from("players"), Value::Array())

        json.insert(String::from("type"), Value::String(String::from(namespace)));

        let matchmaker = &self.matchmaker;

        let data = serializer.serialize(matchmaker)?;

        json.insert(String::from("matchmaker"), data);

        Ok(json!(json))
    }

    pub fn load(value: &Value, serializer_registry: &Registry) -> Result<Arc<Mutex<Self>>, String> {
        let queue_name = value
            .get("name")
            .ok_or("Could not find queue name")?
            .as_str()
            .ok_or("Could not find queue name")?;
        let namespace = value
            .get("type")
            .ok_or("Could not find queue name")?
            .as_str()
            .ok_or("Could not find queue name")?;

        let data = value
            .get("matchmaker")
            .ok_or("Could not find matchmaker data")?;

        let serializer = serializer_registry
            .get(namespace)
            .ok_or(format!("Failed to find serializer for {namespace}"))?;

        let matchmaker = serializer.deserialize(data.clone())?;

        let ticker = Self::new(
            Queue::new(String::from(queue_name)),
            matchmaker,
            Box::new(FakeGame {}),
        );

        Ok(ticker)
    }
}
