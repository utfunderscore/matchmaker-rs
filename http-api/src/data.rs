use common::queue::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueData {
    pub name: String,
    pub entries: Vec<Entry>,
    pub matchmaker: Value,
}
