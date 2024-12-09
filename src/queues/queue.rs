use super::queue_entry::QueueEntry;
use log;
use serde::Serialize;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Queue {
    pub name: String,
    pub in_queue: Vec<QueueEntry>,
    #[serde(skip_serializing)]
    pub players: HashSet<Uuid>,
}

impl Queue {
    pub fn add_team(&mut self, queue_entry: QueueEntry) -> Result<bool, String> {
        log::info!("Adding queue entry to team {:?}", queue_entry);
        if let Some(x) = queue_entry
            .players
            .iter()
            .find(|x| self.players.contains(x))
        {
            log::info!("  - Failed: Already in queue");
            return Err(format!("{x} is already in the queue"));
        }

        let entry_pointer = &queue_entry.players.clone();
        self.in_queue.push(queue_entry);
        for x in entry_pointer {
            self.players.insert(*x);
        }

        log::info!("  - User added to queue");
        Ok(true)
    }

    pub fn remove_team(&mut self, id: &Uuid) -> Result<bool, String> {
        log::info!("Removing queue entry from team {:?}", id);
        if let Some(index) = self.in_queue.iter().position(|x| x.id == *id) {
            let entry = self.in_queue.remove(index);
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
            in_queue: Vec::new(),
            players: HashSet::new(),
        }
    }
}
