use crate::matchmaker;
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id: Uuid,
    players: Vec<Uuid>,
    metadata: Map<String, Value>,
}

impl Entry {
    pub fn new(players: Vec<Uuid>) -> Self {
        Entry {
            id: Uuid::new_v4(),
            players,
            metadata: Map::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn entries(&self) -> &Vec<Uuid> {
        &self.players
    }
}

pub struct Queue {
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
    senders: HashMap<Uuid, Sender<Value>>
}
impl Queue {
    pub fn new(matchmaker: Box<dyn Matchmaker + Send + Sync>) -> Self {
        Queue {
            matchmaker,
            senders: HashMap::new(),
        }
    }

    pub fn from(value: PathBuf) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(value).expect("Failed to read queue file");
        let json_value: Value = serde_json::from_str(&json).expect("Failed to parse JSON");
        let mut queue = Queue::deserialize(json_value)?;
        queue.senders = HashMap::new();
        Ok(queue)
    }

    pub async fn tick(&mut self) -> bool {

        let result = self.matchmaker.matchmake();

        let success = match result {
            MatchmakerResult::Matched(teams) => {
                info!("Matched teams: {:?}", teams);
                for team_id in teams.iter().flatten() {
                    let remove_result = self.leave_queue(
                        team_id,
                        serde_json::to_value(&teams).unwrap_or_default(),
                    );
                    if let Err(e) = remove_result {
                        error!("Failed to remove entry {}: {}", team_id, e);
                    }
                }
                true
            }
            MatchmakerResult::Skip(_) => false,
            MatchmakerResult::Error((err, affected)) => {
                match affected {
                    None => {
                        self.remove_all(Value::String(err));
                    }
                    Some(affected) => {
                        for entry_id in affected {
                            let remove_result =
                                self.leave_queue(&entry_id, Value::String(err.clone()));
                            if let Err(e) = remove_result {
                                error!("Failed to remove entry {}: {}", entry_id, e);
                            }
                        }
                    }
                }
                false
            }
        };

        success
    }

    pub fn remove_all(&mut self, reason: Value) {
        warn!("Removing all entries from queue due to: {}", reason);

        //Drain senders and send the reason
        self.senders.drain().for_each(|(_, sender)| {
           let _ = sender.send(reason.clone()); 
        });
        
    }

    pub fn leave_queue(
        &mut self,
        entry_id: &Uuid,
        result: Value,
    ) -> Result<(), Box<dyn Error>> {

        self.matchmaker.remove_entry(entry_id)?;
        let sender = self.senders.remove(entry_id);

        match sender {
            None => {}
            Some(sender) => {
                let _ = sender.send(result);
            }
        }

        Ok(())
    }

    pub fn remove_entry(&mut self, entry_id: &Uuid) -> Result<(), Box<dyn Error>> {
        let _ = self.matchmaker.remove_entry(entry_id);
        self.senders.remove(entry_id);
        debug!("Removed entry with ID: {}", entry_id);
        
        Ok(())
    }

    pub fn join_queue(
        &mut self,
        entry: Entry,
    ) -> Result<Receiver<Value>, Box<dyn Error>> {
        info!("Joining queue: {:?}", entry);
        let (sender, receiver) = oneshot::channel();
        let id = entry.id();

        self.matchmaker.add_entry(entry)?;
        self.senders.insert(id, sender);
        Ok(receiver)
    }

    pub fn save<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), Box<dyn Error>> {
        let file_path = path.as_ref().join(format!("{}.json", name));
        let json = serde_json::to_string(&self.serialize()?)?;
        Ok(fs::write(file_path, json)?)
    }

    pub fn serialize(&self) -> Result<Value, Box<dyn Error>> {
        matchmaker::serialize(self.matchmaker())
    }

    pub fn deserialize(json: Value) -> Result<Self, Box<dyn Error>> {
        matchmaker::deserialize(json.clone()).map(|matchmaker| Queue {
            matchmaker,
            senders: HashMap::new(),
        })
    }

    pub fn get_entries(&self) -> Vec<&Entry> {
        self.matchmaker.get_entries()
    }

    pub fn matchmaker(&self) -> &Box<dyn Matchmaker + Send + Sync> {
        &self.matchmaker
    }
}
