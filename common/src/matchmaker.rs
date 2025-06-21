use crate::queue_entry::QueueEntry;
use serde_json::Value;
use uuid::Uuid;

pub trait Matchmaker {
    fn matchmake(&self, teams: Vec<QueueEntry>) -> Result<Vec<Vec<Uuid>>, String>;
    
    fn validate_entry(&self, entry: &QueueEntry) -> Result<(), String>;

    fn serialize(&self) -> Result<Value, String>;
}
