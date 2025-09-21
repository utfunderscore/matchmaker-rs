use crate::entry::{Entry, EntryId};
use crate::gamefinder::Game;
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

pub struct Queue {
    pub id: String,
    matchmaker: Box<dyn Matchmaker>,
    entries: HashMap<EntryId, Entry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueueResult {
    pub teams: Vec<Vec<Entry>>,
    pub game: Game,
}

impl QueueResult {
    pub fn new(teams: Vec<Vec<Entry>>, game: Game) -> Self {
        Self { teams, game }
    }
}

impl Queue {
    pub fn new(
        id: String,
        matchmaker: Box<dyn Matchmaker>,
        entries: HashMap<EntryId, Entry>,
    ) -> Self {
        Self {
            id,
            matchmaker,
            entries,
        }
    }

    pub fn tick(&self) -> MatchmakerResult {
        self.matchmaker.matchmake()
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn std::error::Error>> {
        self.matchmaker.add_entry(entry.clone())?;
        self.entries.insert(entry.id, entry);
        Ok(())
    }

    pub fn matchmaker(&self) -> &Box<dyn Matchmaker> {
        &self.matchmaker
    }

    pub fn entries(&self) -> &HashMap<EntryId, Entry> {
        &self.entries
    }

    pub fn has_player(&self, player_id: &Uuid) -> bool {
        self.entries.values().any(|x| x.players.contains(player_id))
    }

    pub fn remove_entry(&mut self, entry_id: &EntryId) -> Option<Entry> {
        let entry = self.entries.remove(entry_id);
        let m_entry = self.matchmaker.remove_entry(entry_id);

        if let Err(err) = m_entry {
            warn!("Failed to remove entry from matchmaker: {}", err);
        }

        entry
    }
}
