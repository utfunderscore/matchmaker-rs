use crate::matchmaker;
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
}

impl Entry {
    pub fn new(players: Vec<Uuid>) -> Self {
        Entry {
            id: Uuid::new_v4(),
            players,
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
    entries: HashMap<Uuid, Entry>,
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
    pending_matches: HashMap<Uuid, Sender<Result<Value, Box<dyn Error + Send + Sync>>>>,
}
impl Queue {
    pub fn new(matchmaker: Box<dyn Matchmaker + Send + Sync>) -> Self {
        Queue {
            entries: HashMap::new(),
            matchmaker,
            pending_matches: HashMap::new(),
        }
    }

    pub fn from(value: PathBuf) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(value).expect("Failed to read queue file");
        let json_value: Value = serde_json::from_str(&json).expect("Failed to parse JSON");
        let mut queue = Queue::deserialize(json_value)?;
        queue.pending_matches = HashMap::new();
        Ok(queue)
    }

    pub async fn tick(&mut self) {
        debug!("Ticking queue");

        let entries: Vec<&Entry> = self.entries.values().collect();

        let result = self.matchmaker.matchmake(entries);

        match result {
            MatchmakerResult::Matched(teams) => {
                info!("Matched teams: {:?}", teams);
                for team_id in teams.iter().flatten() {
                    let remove_result = self.remove_entry(
                        team_id,
                        Some(serde_json::to_value(&teams).unwrap_or_default()),
                    );
                    if let Err(e) = remove_result {
                        error!("Failed to remove entry {}: {}", team_id, e);
                    }
                }
            }
            MatchmakerResult::Skip(_) => {}
            MatchmakerResult::Error((err, affected)) => match affected {
                None => {
                    self.remove_all(Value::String(err));
                    return;
                }
                Some(affected) => {
                    for entry_id in affected {
                        let remove_result =
                            self.remove_entry(&entry_id, Some(Value::String(err.clone())));
                        if let Err(e) = remove_result {
                            error!("Failed to remove entry {}: {}", entry_id, e);
                        }
                    }
                }
            },
        }
    }

    pub fn remove_all(&mut self, reason: Value) {
        warn!("Removing all entries from queue due to: {}", reason);
        let entry_ids: Vec<Uuid> = self.entries.keys().cloned().collect();
        for entry_id in entry_ids {
            if let Err(e) = self.remove_entry(&entry_id, Some(reason.clone())) {
                error!("Failed to remove entry {}: {}", entry_id, e);
            }
        }
    }

    pub fn remove_entry(
        &mut self,
        entry_id: &Uuid,
        result: Option<Value>,
    ) -> Result<(), Box<dyn Error>> {
        self.entries
            .remove(entry_id)
            .ok_or(format!("Entry {} not found", entry_id))?;
        let channel = self
            .pending_matches
            .remove(&entry_id)
            .ok_or(format!("Entry {} not found", entry_id))?;

        if let Some(result) = result {
            channel
                .send(Ok(result))
                .map_err(|_| format!("Failed to send leave message to {}", entry_id))?;
        }

        Ok(())
    }

    pub fn join_queue(
        &mut self,
        entry: Entry,
    ) -> Receiver<Result<Value, Box<dyn Error + Send + Sync>>> {
        debug!("Joining queue");
        let (sender, receiver) = oneshot::channel();
        self.pending_matches.insert(entry.id(), sender);
        self.entries.insert(entry.id(), entry);
        receiver
    }

    pub fn update_matchmaker(
        &mut self,
        matchmaker: Box<dyn Matchmaker + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        self.matchmaker = matchmaker;

        self.entries
            .retain(|_, entry| self.matchmaker.is_valid_entry(entry).is_ok());

        Ok(())
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
            entries: HashMap::new(),
            matchmaker,
            pending_matches: HashMap::new(),
        })
    }

    pub fn get_entries(&self) -> &HashMap<Uuid, Entry> {
        &self.entries
    }

    pub fn matchmaker(&self) -> &Box<dyn Matchmaker + Send + Sync> {
        &self.matchmaker
    }
}
