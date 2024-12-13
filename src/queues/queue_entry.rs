use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QueueEntry {
    pub id: Uuid,
    pub players: Vec<Uuid>,
    pub attributes: Value,
}

impl QueueEntry {
    pub fn new(players: Vec<Uuid>, attributes: Value) -> QueueEntry {
        QueueEntry {
            id: Uuid::new_v4(),
            players,
            attributes,
        }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}

impl Eq for QueueEntry {}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for QueueEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
