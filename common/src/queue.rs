use crate::queue_entry::QueueEntry;
use crate::registry::Registry;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Queue {    #[serde(skip_serializing, default)]
    entries: HashMap<Uuid, QueueEntry>,
    pub matchmaker: String,
}

impl Queue {
    pub fn new(matchmaker: String) -> Self {
        Queue {
            entries: HashMap::new(),
            matchmaker,
        }
    }

    pub fn add_entry(&mut self, entry: QueueEntry, registry: Registry) -> Result<(), String> {
        if self.entries.contains_key(&entry.id) {
            return Err("Entry already exists in the queue".to_string());
        }
        let matchmaker = registry.get_matchmaker(&self.matchmaker).ok_or("Matchmaker not found")?;
        if let Err(e) = matchmaker.is_valid_entry(&entry) {
            return Err(format!("Invalid entry: {}", e));
        }

        self.entries.insert(entry.id, entry);
        Ok(())
    }

    pub fn tick(&mut self, registry: &Registry) -> Result<Vec<Vec<Uuid>>, String> {
        let matchmaker = registry.get_matchmaker(&self.matchmaker).ok_or("")?;

        let entries: Vec<QueueEntry> = self.entries.values().cloned().collect();

        let teams = matchmaker.matchmake(entries)?;

        for team in &teams {
            for entry_id in team {
                self.entries.remove(entry_id);
            }
        }

        Ok(teams)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn success() {}
}
