use crate::matchmaker;
use crate::matchmaker::Matchmaker;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

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

pub struct Queue {
    entries: HashMap<Uuid, Entry>,
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
}
impl Queue {
    pub fn new(matchmaker: Box<dyn Matchmaker + Send + Sync>) -> Self {
        Queue {
            entries: HashMap::new(),
            matchmaker,
        }
    }

    pub fn from(value: PathBuf) -> Result<Self, String> {
        let json = std::fs::read_to_string(value).expect("Failed to read queue file");
        let json_value: Value = serde_json::from_str(&json).expect("Failed to parse JSON");
        Queue::deserialize(json_value)
    }

    pub fn tick(&mut self) -> Result<(), String> {
        todo!()
    }
    
    pub fn save<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), String> {
        let file_path = path.as_ref().join(format!("{}.json", name));
        let json = serde_json::to_string(&self.serialize()?).map_err(|e| e.to_string())?;
        std::fs::write(file_path, json).map_err(|e| e.to_string())
    }

    pub fn serialize(&self) -> Result<Value, String> {
        matchmaker::serialize(self.matchmaker())
    }

    pub fn deserialize(json: Value) -> Result<Self, String> {
        matchmaker::deserialize(json.clone()).map(Queue::new)
    }
    
    pub fn add_entry(&mut self, queue_entry: Entry) -> Result<(), String> {
        self.matchmaker.is_valid_entry(&Entry::new(vec![]))?;

        self.entries.insert(queue_entry.id(), queue_entry);

        Ok(())
    }

    pub fn remove_entry(&mut self, entry_id: Uuid) -> Result<(), String> {
        if self.entries.remove(&entry_id).is_none() {
            return Err(format!("Entry with ID {} not found", entry_id));
        }
        Ok(())
    }

    pub fn get_entries(&self) -> Vec<Entry> {
        self.entries.values().cloned().collect()
    }

    pub fn matchmaker(&self) -> &Box<dyn Matchmaker + Send + Sync> {
        &self.matchmaker
    }
}
