use common::entry::Entry;
use common::gamefinder::Game;
use common::matchmaker::Matchmaker;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueData {
    name: String,
    entries: Vec<Entry>,
    matchmaker: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueError {
    error: String,
}

impl QueueError {
    pub fn new(error: String) -> Self {
        Self { error }
    }
}

impl QueueData {
    pub fn new(name: String, entries: Vec<Entry>, matchmaker: Value) -> Self {
        QueueData {
            name,
            entries,
            matchmaker,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueJoinRequest {
    pub id: Uuid,
    pub players: Vec<Uuid>,
    pub metadata: Map<String, Value>,
}
