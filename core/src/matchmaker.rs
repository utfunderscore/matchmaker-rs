use uuid::Uuid;
use crate::queue_entry;

pub trait Matchmaker<T> where T : queue_entry::QueueEntry {
    
    fn matchmake(&self, teams: &Vec<T>) -> Result<Vec<Vec<Uuid>>, String>;
    
}