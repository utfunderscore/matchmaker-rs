use super::queue_entry::QueueEntry;
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
        if let Some(x) = queue_entry
            .players
            .iter()
            .find(|x| self.players.contains(x))
        {
            return Err(format!("{x} is already in the queue"));
        }

        let entry_pointer = &queue_entry.players.clone();
        self.in_queue.push(queue_entry);
        for x in entry_pointer {
            self.players.insert(*x);
        }

        Ok(true)
    }

    pub fn remove_team(&mut self, id: Uuid) -> Result<bool, String> {
        if let Some(index) = self.in_queue.iter().position(|x| x.id == id) {
            let entry = self.in_queue.remove(index);
            for x in entry.players {
                self.players.remove(&x);
            }

            Ok(true)
        } else {
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
