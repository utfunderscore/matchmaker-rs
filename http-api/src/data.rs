use common::entry::Entry;
use common::gamefinder::Game;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;
use common::matchmaker::Matchmaker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueData {
    name: String,
    entries: Vec<Entry>,
    matchmaker: Value,
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
