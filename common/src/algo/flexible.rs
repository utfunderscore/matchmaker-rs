use crate::matchmaker::MatchmakerResult::{Matched, Skip};
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use crate::queue::Entry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexibleMatchMaker {
    target_team_size: i16,
    min_entry_size: i16,
    max_entry_size: i16,
    num_teams: i16,
    #[serde(skip)]
    valid_team_compositions: Vec<Vec<i16>>,
    #[serde(skip)]
    teams_by_size: Vec<Vec<Uuid>>, // the index of the first array is the team size
    #[serde(skip)]
    teams: HashMap<Uuid, Entry>,
}

impl FlexibleMatchMaker {
    pub fn new(
        target_team_size: i16,
        min_entry_size: i16,
        max_entry_size: i16,
        num_teams: i16,
    ) -> Result<Self, Box<dyn Error>> {
        let valid_team_compositions = find_unique_addends(target_team_size)?;

        Ok(FlexibleMatchMaker {
            target_team_size,
            min_entry_size,
            max_entry_size,
            num_teams,
            valid_team_compositions,
            teams_by_size: Vec::new(),
            teams: HashMap::new(),
        })
    }
}

impl FlexibleMatchMaker {
    pub fn deserialize(value: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, Box<dyn Error>> {
        let mut matchmaker: FlexibleMatchMaker = serde_json::from_value(value)?;

        matchmaker.valid_team_compositions = find_unique_addends(matchmaker.target_team_size)?;

        Ok(Box::new(matchmaker))
    }
}
impl Matchmaker for FlexibleMatchMaker {
    fn get_type_name(&self) -> String {
        "flexible".to_string()
    }

    fn matchmake(&self) -> MatchmakerResult {
        let total_players: usize = self.teams_by_size.iter().map(|team| team.len()).sum();

        if (total_players as i32) < (self.target_team_size as i32) * (self.num_teams as i32) {
            return Skip("Not enough players to form a match".to_string());
        }

        let mut results: Vec<Vec<Uuid>> = Vec::new();

        for _ in 0..self.num_teams {
            let composition = self.valid_team_compositions.iter().find(|sizes| {
                sizes.iter().all(|&sz| {
                    self.teams_by_size
                        .get(sz as usize)
                        .map_or(false, |dq| !dq.is_empty())
                })
            });

            let sizes = match composition {
                Some(c) => c,
                None => return Skip("No valid team composition found".to_string()),
            };

            let mut index_tracker: Vec<usize> = Vec::new();

            let mut picked: Vec<Uuid> = Vec::with_capacity(sizes.len());
            for &sz in sizes {
                // unwrap is safe because we checked availability above
                let queue = self.teams_by_size.get(sz as usize).unwrap();
                let index = index_tracker.get(sz as usize).unwrap_or(&0usize);

                picked.push(queue.get(*index).unwrap().clone());

                index_tracker.push(*index + 1);
            }

            results.push(picked);
        }

        Matched(results)
    }

    // fn is_valid_entry(&self, entry: &Entry) -> Result<(), Box<dyn Error>> {
    //
    //
    //     Ok(())
    // }

    fn serialize(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(self)?)
    }

    fn remove_all(&mut self) -> Vec<Entry> {
        self.teams_by_size.clear();
        self.teams.drain().map(|(_, entry)| entry).collect()
    }

    fn get_entries(&self) -> Vec<&Entry> {
        self.teams.values().collect()
    }

    fn remove_entry(&mut self, entry_id: &Uuid) -> Result<Entry, Box<dyn Error>> {
        let entry = self.teams.remove(entry_id).ok_or("Entry not found")?;
        let size = entry.entries().len();

        let teams = self.teams_by_size.get_mut(size).unwrap();
        teams.retain(|id| id != entry_id);

        Ok(entry)
    }

    fn get_entry(&self, entry_id: &Uuid) -> Option<&Entry> {
        self.teams.get(entry_id)
    }

    fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn Error>> {
        if entry.entries().len() < self.min_entry_size as usize {
            return Err(format!(
                "Entry size {} is less than minimum required size {}",
                entry.entries().len(),
                self.min_entry_size
            )
            .into());
        }

        if entry.entries().len() > self.max_entry_size as usize {
            return Err(format!(
                "Entry size {} exceeds maximum allowed size {}",
                entry.entries().len(),
                self.max_entry_size
            )
            .into());
        }

        let size = entry.entries().len();

        match self.teams_by_size.get_mut(size) {
            None => {
                self.teams_by_size.push(vec![entry.id()]);
            }
            Some(teams) => {
                teams.push(entry.id());
            }
        }

        self.teams.insert(entry.id(), entry);

        Ok(())
    }
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
struct BacktrackState {
    remaining: i16,
    current_combination: Vec<i16>,
    start: i16,
}

pub fn find_unique_addends(target: i16) -> Result<Vec<Vec<i16>>, String> {
    if target <= 0 {
        return Err("Target must be a positive integer".to_string());
    }

    let mut result: Vec<Vec<i16>> = Vec::new();

    let mut stack: VecDeque<BacktrackState> = VecDeque::new();
    stack.push_front(BacktrackState {
        remaining: target,
        current_combination: vec![],
        start: 1,
    });

    while let Some(state) = stack.pop_front() {
        let BacktrackState {
            remaining,
            current_combination,
            start,
        } = state;

        if remaining == 0 {
            result.push(current_combination.clone());
            continue;
        }

        if remaining < 0 {
            continue;
        }

        for i in start..=remaining {
            let mut new_combination = current_combination.clone();
            new_combination.push(i);

            stack.push_front(BacktrackState {
                remaining: remaining - i,
                current_combination: new_combination,
                start: i,
            });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::debug;

    #[test]
    fn test_find_unique_addends() {
        let target = 5;

        //[[5], [2, 3], [1, 4], [1, 2, 2], [1, 1, 3], [1, 1, 1, 2], [1, 1, 1, 1, 1]]
        debug!("{:?}", find_unique_addends(target));

        assert_eq!(
            find_unique_addends(target).unwrap(),
            vec![
                vec![5],
                vec![2, 3],
                vec![1, 4],
                vec![1, 2, 2],
                vec![1, 1, 3],
                vec![1, 1, 1, 2],
                vec![1, 1, 1, 1, 1]
            ]
        );
    }

    #[test]
    fn test_negative_addends() {
        let target = -5;

        // Should return an error
        assert!(find_unique_addends(target).is_err());
    }

    #[test]
    fn test_construct_success() {
        let matchmaker = FlexibleMatchMaker::new(5, 1, 5, 2);

        assert!(matchmaker.is_ok());
    }

    #[test]
    fn test_construct_failure() {
        let matchmaker = FlexibleMatchMaker::new(-5, 1, 5, 2);

        assert!(matchmaker.is_err());
    }

    #[test]
    fn test_matchmake_success() {
        let mut matchmaker = FlexibleMatchMaker::new(1, 1, 1, 2).unwrap();

        let team1 = Entry::new(Uuid::new_v4(), vec![Uuid::new_v4()]);
        let team2 = Entry::new(Uuid::new_v4(), vec![Uuid::new_v4()]);

        matchmaker.add_entry(team1).unwrap();
        matchmaker.add_entry(team2).unwrap();

        let result = matchmaker.matchmake();

        assert!(result.is_matched());
    }

    #[test]
    fn test_matchmake_not_enough_players() {
        let mut matchmaker = FlexibleMatchMaker::new(5, 1, 5, 2).unwrap();

        let team1 = Entry::new(Uuid::new_v4(), vec![Uuid::new_v4()]);

        matchmaker.add_entry(team1).unwrap();

        let result: MatchmakerResult = matchmaker.matchmake();

        assert!(result.is_skip());
        let error = result.unwrap_skip();

        assert_eq!(error, "Not enough players to form a match".to_string());
    }
}
