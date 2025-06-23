use crate::matchmaker::Matchmaker;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id: uuid::Uuid,
    pub(crate) entries: Vec<Uuid>,
}

impl Entry {
    pub fn new(players: Vec<Uuid>) -> Self {
        Entry {
            id: Uuid::new_v4(),
            entries: players
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entries(&self) -> &Vec<Uuid> {
        &self.entries
    }
}

#[derive(Debug, Serialize)]
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

    fn remove_entry(&mut self, entry_id: Uuid) -> Result<(), String> {
        if self.entries.remove(&entry_id).is_none() {
            return Err(format!("Entry with ID {} not found", entry_id));
        }
        Ok(())
    }

    fn get_entries(&self) -> Vec<Entry> {
        self.entries.values().cloned().collect()
    }

    fn serialize(&self) -> Result<Value, String> {
        let mut state = serde_json::Map::new();
        state.insert("matchmaker".to_string(), self.matchmaker.serialize()?);
        
        let entries= serde_json::to_value(&self.entries).map_err(|e| e.to_string())?;
        state.insert("entries".to_string(), entries);

        Ok(Value::Object(state))
    }
}

pub trait QueueTrait: Send + Sync {
    fn tick(&mut self) -> Result<(), String>;

    fn add_entry(&mut self, queue_entry: Entry) -> Result<(), String>;

    fn remove_entry(&mut self, entry_id: Uuid) -> Result<(), String>;

    fn get_entries(&self) -> Vec<Entry>;
    
    fn serialize(&self) -> Result<Value, String>;
}
