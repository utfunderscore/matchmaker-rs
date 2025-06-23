use crate::matchmaker::Matchmaker;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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
    
    pub fn tick(&mut self) -> Result<(), String> {
        todo!()
    }

    pub fn add_entry(&mut self, queue_entry: Entry) -> Result<(), String> {
        self.matchmaker.is_valid_entry(&Entry::new(vec![]))?;

        self.entries.insert(queue_entry.id(), queue_entry);

        Ok(())
    }

    pub fn serialize(&self) -> Result<Value, String> {
        let entries_json = serde_json::to_value(&self.entries).map_err(|e| e.to_string())?;
        let matchmaker_json = self.matchmaker.serialize()?;

        let mut result = serde_json::Map::new();
        result.insert("entries".to_string(), entries_json);
        result.insert("matchmaker".to_string(), matchmaker_json);
        Ok(Value::Object(result))
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
}
