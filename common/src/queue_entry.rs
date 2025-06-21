use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct QueueEntry {
    pub id: Uuid,
    pub entries: Vec<Uuid>,
    pub metadata: Map<String, Value>,
}

impl QueueEntry {
    pub fn new(entries: Vec<Uuid>, metadata: Map<String, Value>) -> Self {
        QueueEntry {
            id: Uuid::new_v4(),
            entries,
            metadata,
        }
    }
}
