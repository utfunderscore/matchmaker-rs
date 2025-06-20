use crate::queue_entry::QueueEntry;
use uuid::Uuid;

pub trait Matchmaker {
    fn matchmake(&self, teams: Vec<QueueEntry>) -> Result<Vec<Vec<Uuid>>, String>;
}
