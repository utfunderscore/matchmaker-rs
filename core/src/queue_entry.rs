use std::hash::Hash;
use uuid::Uuid;


// Represents a team or player in a queue.

pub trait QueueEntry {
    fn id(&self) -> Uuid;
    fn players(&self) -> &Vec<Uuid>;
}

impl Hash for dyn QueueEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}


