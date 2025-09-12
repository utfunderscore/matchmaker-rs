use crate::matchmaker::MatchmakerResult::{Matched, Skip};
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::ops::Sub;
use tracing::warn;
use crate::entry::{Entry, EntryId};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EloMatchmaker {
    scaling_factor: f64,
    team_size: i64,
    max_skill_diff: i64,
    #[serde(skip)]
    elo_map: BTreeMap<i64, HashSet<EntryId>>,
    #[serde(skip)]
    entries: HashMap<EntryId, Entry>,
}

impl EloMatchmaker {

    fn get_elo(entry: &Entry) -> Option<i64> {
        entry.metadata.get("elo").map(|v| v.as_i64()).flatten()
    }

    fn get_elo_range(&self, entry: &Entry) -> Result<(i64, i64), &'static str> {
        let elo = entry.metadata.get("elo").map(|v| v.as_i64()).flatten().ok_or("Entry has no elo")?;

        // time since queued in seconds
        let duration = chrono::Utc::now().sub(entry.time_queued).as_seconds_f64();

        let incr = (duration * self.scaling_factor) as i64;
        Ok((elo - incr, elo + incr))
    }

    pub fn deserialize(value: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, Box<dyn Error>> {
        let matchmaker: EloMatchmaker = serde_json::from_value(value)?;
        Ok(Box::new(matchmaker))
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
                warn!("Entry {:?} has no elo, which should never happen", id);
                continue;
            }
            let elo = elo_opt.unwrap();
            let Ok((lower, upper)) = self.get_elo_range(entry) else {
                warn!("Entry {:?} has no elo range, which should never happen", id);
                continue;
            };

            let nearby = self.elo_map.range(lower..=upper);

            let mut closest_candidate: Option<EntryId> = None;
            let mut min_diff = i64::MAX;

            for (&nearby_elo, ids) in nearby {
                for &candidate_id in ids {
                    if candidate_id == *id {
                        continue; // Don't match with self
                    }

                    let diff = (nearby_elo - elo).abs();
                    if diff > self.max_skill_diff {
                        continue; // Skip if difference is too high
                    }

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
        serde_json::to_value(self).map_err(|x| x.into())
    }

    fn remove_all(&mut self) -> Vec<Entry> {
        self.elo_map.clear();
        self.entries.drain().map(|(_, v)| v).collect()
    }

    fn get_entries(&self) -> Vec<&Entry> {
        self.entries.values().collect()
    }

    fn remove_entry(&mut self, entry_id: &EntryId) -> Result<Entry, Box<dyn Error>> {
        let entry = self.entries.remove(entry_id).ok_or("Entry not found")?;
        let elo = EloMatchmaker::get_elo(&entry).ok_or("Entry has no elo")?;

        if let Some(entries) = self.elo_map.get_mut(&elo) {
            entries.remove(entry_id);
        }
        Ok(entry)
    }

    fn get_entry(&self, entry_id: &EntryId) -> Option<&Entry> {
        self.entries.get(entry_id)
    }

    fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn Error>> {
        if entry.players.len() != self.team_size as usize {
            return Err("Entry has wrong team size".into());
        }
        let elo = Self::get_elo(&entry).ok_or("Entry has no elo")?;

        let id = entry.id;
        self.entries.insert(id, entry);
        self.elo_map.entry(elo).or_default().insert(id);

        Ok(())
    }
}
