use crate::gamefinder::Game;
use crate::matchmaker;
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id: Uuid,
    players: Vec<Uuid>,
    pub time_queued: u64,
    pub metadata: Map<String, Value>,
}

impl Entry {
    pub fn new(id: Uuid, players: Vec<Uuid>, metadata: Map<String, Value>) -> Self {
        Entry {
            id,
            players,
            metadata,
            time_queued: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entries(&self) -> &Vec<Uuid> {
        &self.players
    }
}

pub enum QueueResult {
    Success(Vec<Vec<Entry>>, Game),
    Error(String),
}

pub struct Queue {
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
    senders: HashMap<Uuid, Sender<QueueResult>>,
}

impl Queue {
    pub async fn tick(&mut self) -> MatchmakerResult {
        self.matchmaker.matchmake()
    }

    pub fn join_queue(&mut self, entry: Entry) -> Result<Receiver<QueueResult>, Box<dyn Error>> {
        info!("Joining queue: {:?}", entry);
        let (sender, receiver) = oneshot::channel();
        let id = entry.id();

        self.matchmaker.add_entry(entry)?;
        self.senders.insert(id, sender);
        Ok(receiver)
    }

    pub fn remove_entry(&mut self, entry_id: &Uuid) -> Result<Entry, Box<dyn Error>> {
        info!("Removing entry with ID: {}", entry_id);
        self.matchmaker.remove_entry(entry_id)
    }

    pub fn remove_sender(&mut self, entry_id: &Uuid) -> Option<Sender<QueueResult>> {
        self.senders.remove(entry_id)
    }

    pub fn remove_all(&mut self) -> Vec<Entry> {
        self.matchmaker.remove_all()
    }

    pub fn get_entries(&self) -> Vec<&Entry> {
        self.matchmaker.get_entries()
    }

    pub fn get_entry(&self, id: &Uuid) -> Option<&Entry> {
        self.get_entries()
            .into_iter()
            .find(|entry| entry.id() == *id)
    }

    pub fn contains_player(&self, player: &Uuid) -> bool {
        self.get_entries()
            .iter()
            .any(|x| x.players.contains(player))
    }

    pub fn matchmaker(&self) -> &Box<dyn Matchmaker + Send + Sync> {
        &self.matchmaker
    }
}

// Creation, serialization, and deserialization methods for Queue
impl Queue {
    pub fn new(matchmaker: Box<dyn Matchmaker + Send + Sync>) -> Self {
        Queue {
            matchmaker,
            senders: HashMap::new(),
        }
    }

    pub fn from(value: PathBuf) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(value).expect("Failed to read queue file");
        let json_value: Value = serde_json::from_str(&json).expect("Failed to parse JSON");
        let mut queue = Queue::deserialize(json_value)?;
        queue.senders = HashMap::new();
        Ok(queue)
    }
    pub fn save<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), Box<dyn Error>> {
        let file_path = path.as_ref().join(format!("{name}.json"));
        let json = serde_json::to_string(&self.serialize()?)?;
        Ok(fs::write(file_path, json)?)
    }

    pub fn serialize(&self) -> Result<Value, Box<dyn Error>> {
        matchmaker::serialize(self.matchmaker())
    }

    pub fn deserialize(json: Value) -> Result<Self, Box<dyn Error>> {
        matchmaker::deserialize(json.clone()).map(|matchmaker| Queue {
            matchmaker,
            senders: HashMap::new(),
        })
    }
}
