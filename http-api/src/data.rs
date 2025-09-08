use common::queue::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
