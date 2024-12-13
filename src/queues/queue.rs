use super::queue_entry::QueueEntry;
use log;
use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Queue {
    pub name: String,
    pub entries: Vec<QueueEntry>,
    pub players: HashSet<Uuid>,
}

impl Queue {
    pub fn add_team(&mut self, queue_entry: QueueEntry) -> Result<bool, String> {
        log::info!("Adding queue entry to team {queue_entry:?}");
        if self.entries.iter().any(|x| x.id == queue_entry.id) {
            return Err(String::from("Entry already exists with that id"));
        }

        if let Some(x) = queue_entry
            .players
            .iter()
            .find(|x| self.players.contains(x))
        {
            log::info!("  - Failed: Already in queue");
            return Err(format!("{x} is already in the queue"));
        }

        let entry_pointer = &queue_entry.players.clone();
        self.entries.push(queue_entry);
        for x in entry_pointer {
            self.players.insert(*x);
        }

        log::info!("  - User added to queue");
        Ok(true)
    }

    pub fn remove_team(&mut self, id: &Uuid) -> Result<bool, String> {
        log::info!("Removing queue entry from team {id:?}");
        if let Some(index) = self.entries.iter().position(|x| x.id == *id) {
            let entry = self.entries.remove(index);
            for x in entry.players {
                self.players.remove(&x);
            }

            Ok(true)
        } else {
            log::info!("  - User removed from queue");
            Err(String::from("Team is not currently in the queue"))
        }
    }

    pub fn new(name: String) -> Self {
        Queue {
            name,
            entries: Vec::new(),
            players: HashSet::new(),
        }
    }
}
