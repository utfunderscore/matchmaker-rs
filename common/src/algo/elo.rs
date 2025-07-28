use crate::matchmaker::MatchmakerResult::{Matched, Skip};
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use crate::queue::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use tracing::warn;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct EloMatchmaker {
    scaling_factor: f32,
    #[serde(skip)]
    elo_map: BTreeMap<i64, HashSet<Uuid>>,
    #[serde(skip)]
    entries: HashMap<Uuid, Entry>,
}

impl EloMatchmaker {
    fn get_elo(entry: &Entry) -> Option<i64> {
        entry.metadata.get("elo").map(|v| v.as_i64()).flatten()
    }

    fn get_elo_range(&self, queue_time: u64, elo: i64) -> (i64, i64) {
        let incr = (queue_time as f32 * self.scaling_factor) as i64;
        (elo - incr, elo + incr)
    }
}

impl Matchmaker for EloMatchmaker {
    fn get_type_name(&self) -> String {
        String::from("elo")
    }

    fn matchmake(&self) -> MatchmakerResult {
        for (id, entry) in &self.entries {
            let elo_opt = Self::get_elo(entry);
            if let None = elo_opt {
                warn!("Entry {} has no elo, which should never happen", id);
                continue;
            }
            let elo = elo_opt.unwrap();
            let (lower, upper) = self.get_elo_range(entry.time_queued, elo);
            let nearby = self.elo_map.range(lower..=upper);

            let mut closest_candidate: Option<Uuid> = None;
            let mut min_diff = i64::MAX;

            for (&nearby_elo, ids) in nearby {
                for &candidate_id in ids {
                    if candidate_id == *id {
                        continue; // Don't match with self
                    }

                    let diff = (nearby_elo - elo).abs();
                    if diff < min_diff {
                        min_diff = diff;
                        closest_candidate = Some(candidate_id);
                    }
                }
            }
            if let Some(opponent) = closest_candidate {
                return Matched(vec![vec![*id], vec![opponent]]);
            }
        }
        Skip(String::from("No teams found"))
    }

    fn serialize(&self) -> Result<Value, Box<dyn Error>> {
        Ok(Value::default())
    }

    fn remove_all(&mut self) -> Vec<Entry> {
        self.elo_map.clear();
        self.entries.drain().map(|(_, v)| v).collect()
    }

    fn get_entries(&self) -> Vec<&Entry> {
        self.entries.values().collect()
    }

    fn remove_entry(&mut self, entry_id: &Uuid) -> Result<Entry, Box<dyn Error>> {
        let entry = self.entries.remove(entry_id).ok_or("Entry not found")?;
        let elo = EloMatchmaker::get_elo(&entry).ok_or("Entry has no elo")?;

        if let Some(entries) = self.elo_map.get_mut(&elo) {
            entries.remove(entry_id);
        }
        Ok(entry)
    }

    fn get_entry(&self, entry_id: &Uuid) -> Option<&Entry> {
        self.entries.get(entry_id)
    }

    fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn Error>> {
        let elo = Self::get_elo(&entry).ok_or("Entry has no elo")?;

        let id = entry.id();
        self.entries.insert(id, entry);
        self.elo_map.entry(elo).or_default().insert(id);

        Ok(())
    }
}
