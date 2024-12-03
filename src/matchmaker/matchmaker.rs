use crate::matchmaker::addends::addends;
use crate::queues::queue_entry::QueueEntry;
use std::collections::HashMap;
use uuid::Uuid;

pub trait Matchmaker {
    fn matchmake<'a>(&'a self, queue: &'a Vec<QueueEntry>) -> Result<Vec<Vec<Uuid>>, String>;
}

pub struct UnratedMatchmaker {
    pub(crate) team_size: u64,
    pub(crate) number_of_teams: u64,
    pub(crate) addends: Vec<HashMap<u64, u32>>,
}

impl UnratedMatchmaker {
    pub fn new(team_size: u64, number_of_teams: u64) -> Box<Self> {
        Box::new(UnratedMatchmaker {
            team_size,
            number_of_teams,
            addends: addends::find_associate_addends(team_size),
        })
    }
}

impl Matchmaker for UnratedMatchmaker {
    fn matchmake<'a>(&'a self, in_queue: &'a Vec<QueueEntry>) -> Result<Vec<Vec<Uuid>>, String> {
        let mut queue_by_size: HashMap<u64, Vec<&QueueEntry>> = HashMap::new();

        for entry in in_queue {
            let current_entries = queue_by_size.get_mut(&(entry.players.len() as u64));
            if current_entries.is_some() {
                current_entries.unwrap().push(entry);
            } else {
                queue_by_size.insert(entry.players.len() as u64, Vec::from([entry]));
            }
        }

        let mut teams: Vec<Vec<Uuid>> = Vec::new();

        for x in 0..self.number_of_teams {
            let addend = find_valid_addend(&self.addends, &queue_by_size);
            if addend.is_none() {
                return Err(String::from(
                    "Unable to build the required amount of teams.",
                ));
            }
            let addend = addend.unwrap();

            let mut team = Vec::<Uuid>::new();

            for (entry_size, number_of_teams) in addend {
                let teams_ref_opt = queue_by_size.get_mut(entry_size);
                if teams_ref_opt.is_none() {
                    return Err(String::from(
                        "Unable to build the required amount of teams.",
                    ));
                }
                let teams_ref = teams_ref_opt.unwrap();

                let number_of_teams_usize = *number_of_teams as usize;

                let mut entries: Vec<Uuid> = Vec::new();

                for i in 0..*number_of_teams {
                    match teams_ref.pop() {
                        Some(x) => entries.push(x.id),
                        None => {
                            return Err(String::from(
                                "Unable to build the required amount of teams.",
                            ))
                        }
                    }
                }

                if entries.len() != number_of_teams_usize {
                    return Err(String::from(
                        "Unable to build the required amount of teams.",
                    ));
                }

                team.extend(entries);
            }

            teams.push(team);
        }

        Ok(teams)
    }
}

fn find_valid_addend<'a>(
    addends: &'a [HashMap<u64, u32>],
    queue_by_size: &HashMap<u64, Vec<&QueueEntry>>,
) -> Option<&'a HashMap<u64, u32>> {
    if let Some(x) = addends.iter().next() {
        for (ref_key, ref_needed_players) in x {
            let in_queue_by_size = queue_by_size.get(ref_key).unwrap_or(&Vec::new()).len() as u32;

            if &in_queue_by_size < ref_needed_players {
                return None;
            }
        }
        return Some(x);
    }

    None
}
