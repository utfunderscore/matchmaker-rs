use crate::matchmaker::Matchmaker;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use crate::matchmaker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id: Uuid,
    pub(crate) entries: Vec<Uuid>,
}

impl Entry {
    pub fn new(players: Vec<Uuid>) -> Self {
        Entry {
            id: Uuid::new_v4(),
            entries: players,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entries(&self) -> &Vec<Uuid> {
        &self.entries
    }
}

#[derive(Debug)]
pub struct Queue<T>
where
    T: Matchmaker + ?Sized + Send + Sync,
{
    entries: HashMap<Uuid, Entry>,
    matchmaker: Box<T>,
}

impl<T> Queue<T>
where
    T: Matchmaker + ?Sized + Send + Sync,
{
    pub fn new(matchmaker: Box<T>) -> Self {
        Queue {
            entries: HashMap::new(),
            matchmaker,
        }
    }

}

impl<T> QueueTrait for Queue<T>
where
    T: Matchmaker + ?Sized + Send + Sync,
{
    fn tick(&mut self) -> Result<(), String> {
        todo!()
    }

    fn add_entry(&mut self, queue_entry: Entry) -> Result<(), String> {
        self.matchmaker.is_valid_entry(&Entry::new(vec![]))?;

        self.entries.insert(queue_entry.id(), queue_entry);

        Ok(())
    }

    fn serialize(&self) -> Result<Value, String> {
        let entries_json = serde_json::to_value(&self.entries).map_err(|e| e.to_string())?;
        let matchmaker_json = self.matchmaker.serialize()?;

        let mut result = serde_json::Map::new();
        result.insert("entries".to_string(), entries_json);
        result.insert("matchmaker".to_string(), matchmaker_json);
        Ok(Value::Object(result))
    }


    fn remove_entry(&mut self, entry_id: Uuid) -> Result<(), String> {
        if self.entries.remove(&entry_id).is_none() {
            return Err(format!("Entry with ID {} not found", entry_id));
        }
        Ok(())
    }

    fn get_entries(&self) -> Vec<Entry> {
        self.entries.values().cloned().collect()
    }
}

pub trait QueueTrait: Send + Sync {
    fn tick(&mut self) -> Result<(), String>;

    fn add_entry(&mut self, queue_entry: Entry) -> Result<(), String>;

    fn serialize(&self) -> Result<Value, String>;

    fn remove_entry(&mut self, entry_id: Uuid) -> Result<(), String>;

    fn get_entries(&self) -> Vec<Entry>;
}
