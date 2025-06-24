use serde::{Deserialize, Serialize};
use serde_json::Value;
use common::queue::Entry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueData {
    pub name: String,
    pub entries: Vec<Entry>,
    pub matchmaker: Value
}
