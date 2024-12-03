use rocket::serde::Serialize;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct QueueEntry {
    pub id: Uuid,
    pub players: Vec<Uuid>,
}

impl QueueEntry {
    pub fn from_vec(players: Vec<Uuid>) -> QueueEntry {
        QueueEntry {
            id: Uuid::new_v4(),
            players,
        }
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
        self.id.hash(state)
    }
}
