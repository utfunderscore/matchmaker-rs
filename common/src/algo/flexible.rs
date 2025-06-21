use crate::codec::MatchmakerDeserializer;
use crate::matchmaker::Matchmaker;
use crate::queue_entry::QueueEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct FlexibleMatchMaker {
    target_team_size: i16,
    min_entry_size: i16,
    max_entry_size: i16,
    num_teams: i16,
    #[serde(skip)]
    valid_team_compositions: Vec<Vec<i16>>,
}

impl FlexibleMatchMaker {
    pub fn new(
        target_team_size: i16,
        min_entry_size: i16,
        max_entry_size: i16,
        num_teams: i16,
    ) -> Result<Self, String> {
        let valid_team_compositions = find_unique_addends(target_team_size)?;

        Ok(FlexibleMatchMaker {
            target_team_size,
            min_entry_size,
            max_entry_size,
            num_teams,
            valid_team_compositions,
        })
    }



}

pub static DESERIALIZER: MatchmakerDeserializer =
    |settings: Value| {
        let mut matchmaker =  serde_json::from_value::<FlexibleMatchMaker>(settings).map_err(|e| format!("Error parsing matchmaker settings: {}", e))?;
        matchmaker.valid_team_compositions = find_unique_addends(matchmaker.target_team_size)?;
        Ok(Box::new(matchmaker))
    };

impl Matchmaker for FlexibleMatchMaker {
    fn matchmake(&self, teams: Vec<QueueEntry>) -> Result<Vec<Vec<Uuid>>, String> {
        let total_players: usize = teams.iter().map(|team| team.entries.len()).sum();

        if (total_players as i32) < (self.target_team_size as i32) * (self.num_teams as i32) {
            return Err("Not enough players to form a match".to_string());
        }

        let mut teams_by_size = teams.iter().fold(HashMap::new(), |mut acc, team| {
            let size = team.entries.len() as i16;
            acc.entry(size).or_insert_with(Vec::new).push(team.id);
            acc
        });

        let mut results: Vec<Vec<Uuid>> = Vec::new();

        for _ in 0..self.num_teams {
            let composition = self.valid_team_compositions.iter().find(|sizes| {
                sizes
                    .iter()
                    .all(|&sz| teams_by_size.get(&sz).map_or(false, |dq| !dq.is_empty()))
            });

            let sizes = match composition {
                Some(c) => c,
                None => return Err("No valid team composition found".to_string()),
            };

            let mut picked: Vec<Uuid> = Vec::with_capacity(sizes.len());
            for &sz in sizes {
                // unwrap is safe because we checked availability above
                let queue = teams_by_size.get_mut(&sz).unwrap();
                picked.push(queue.pop().unwrap());
            }

            results.push(picked);
        }

        Ok(results)
    }

    fn serialize(&self) -> Result<Value, String> {
        serde_json::to_value(self).map_err(|e| format!("Error serializing matchmaker settings: {}", e))
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
    use serde_json::Map;

    #[test]
    fn test_find_unique_addends() {
        let target = 5;

        //[[5], [2, 3], [1, 4], [1, 2, 2], [1, 1, 3], [1, 1, 1, 2], [1, 1, 1, 1, 1]]
        println!("{:?}", find_unique_addends(target));

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
        let matchmaker = FlexibleMatchMaker::new(1, 1, 1, 2).unwrap();

        let team1 = QueueEntry::new(vec![Uuid::new_v4()], Map::new());
        let team2 = QueueEntry::new(vec![Uuid::new_v4()], Map::new());

        let teams = vec![team1, team2];

        let result = matchmaker.matchmake(teams);

        assert!(result.is_ok());
    }

    #[test]
    fn test_matchmake_not_enough_players() {
        let matchmaker = FlexibleMatchMaker::new(5, 1, 5, 2).unwrap();

        let team1 = QueueEntry::new(vec![Uuid::new_v4()], Map::new());
        let teams = vec![team1];

        let result = matchmaker.matchmake(teams);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough players to form a match");
    }
}
