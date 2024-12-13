use crate::matchmaker::addends;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_ticker::QueueTicker;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub trait Matchmaker {
    fn namespace(&self) -> &str;
    fn matchmake(&self, queue: &[QueueEntry]) -> Result<Vec<Vec<Uuid>>, String>;
    fn validate_entry(&self, queue_entry: &QueueEntry) -> Result<bool, String>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize, Deserialize)]
pub struct Unrated {
    pub(crate) team_size: u64,
    pub(crate) number_of_teams: u64,
    pub(crate) addends: Vec<HashMap<u64, u64>>,
}

impl Unrated {
    #[allow(clippy::unnecessary_box_returns)]
    pub fn new(team_size: u64, number_of_teams: u64) -> Box<Self> {
        Box::new(Unrated {
            team_size,
            number_of_teams,
            addends: addends::find(team_size),
        })
    }
    pub fn create_unrated_queue(
        name: String,
        body: &Value,
    ) -> Result<Arc<Mutex<QueueTicker>>, String> {
        let team_size = body
            .get("team_size")
            .ok_or("Missing json attribute 'team_size'")?
            .as_u64()
            .ok_or("'team_size' must be an integer")?;

        let number_of_teams = body
            .get("number_of_teams")
            .ok_or("Missing json attribute 'number_of_teams'")?
            .as_u64()
            .ok_or("'number_of_teams' must be an integer")?;

        Ok(QueueTicker::new(
            Queue::new(name),
            Unrated::new(team_size, number_of_teams),
            Box::new(crate::FakeGame {}),
        ))
    }
}

impl Matchmaker for Unrated {
    fn namespace(&self) -> &str {
        "unrated"
    }

    #[allow(clippy::cast_possible_truncation)]
    fn matchmake(&self, in_queue: &[QueueEntry]) -> Result<Vec<Vec<Uuid>>, String> {
        let mut queue_by_size: HashMap<u64, Vec<&QueueEntry>> = HashMap::new();

        for entry in in_queue {
            let current_entries = queue_by_size.get_mut(&(entry.players.len() as u64));
            if let Some(item) = current_entries {
                item.push(entry);
            } else {
                queue_by_size.insert(entry.players.len() as u64, Vec::from([entry]));
            }
        }

        let mut teams: Vec<Vec<Uuid>> = Vec::new();

        for _x in 0..self.number_of_teams {
            let addend = find_valid_addend(&self.addends, &queue_by_size);
            if addend.is_none() {
                return Err(String::from(
                    "Unable to build the required amount of teams (1).",
                ));
            }
            let addend = addend.unwrap();

            let mut team = Vec::<Uuid>::new();

            for (entry_size, number_of_teams) in addend {
                let teams_ref_opt = queue_by_size.get_mut(entry_size);
                if teams_ref_opt.is_none() {
                    return Err(String::from(
                        "Unable to build the required amount of teams (2).",
                    ));
                }
                let teams_ref = teams_ref_opt.unwrap();

                let number_of_teams_usize = *number_of_teams as usize;

                let mut entries: Vec<Uuid> = Vec::new();

                for _i in 0..*number_of_teams {
                    match teams_ref.pop() {
                        Some(x) => entries.push(x.id),
                        None => {
                            return Err(String::from(
                                "Unable to build the required amount of teams (3).",
                            ));
                        }
                    }
                }

                if entries.len() != number_of_teams_usize {
                    return Err(String::from(
                        "Unable to build the required amount of teams. (4)",
                    ));
                }

                team.extend(entries);
            }

            teams.push(team);
        }

        Ok(teams)
    }

    fn validate_entry(&self, queue_entry: &QueueEntry) -> Result<bool, String> {
        if queue_entry.players.len() as u64 <= self.team_size {
            Ok(true)
        } else {
            error!("Team size cannot exceed {} players", self.team_size);
            Err(format!(
                "Team size cannot exceed {} players",
                self.team_size
            ))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn find_valid_addend<'a>(
    addends: &'a [HashMap<u64, u64>],
    queue_by_size: &HashMap<u64, Vec<&QueueEntry>>,
) -> Option<&'a HashMap<u64, u64>> {
    if let Some(x) = addends.iter().next() {
        for (ref_key, ref_needed_players) in x {
            let in_queue_by_size = queue_by_size.get(ref_key).unwrap_or(&Vec::new()).len() as u64;

            if &in_queue_by_size < ref_needed_players {
                return None;
            }
        }
        return Some(x);
    }

    None
}
