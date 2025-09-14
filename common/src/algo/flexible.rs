use crate::entry::{Entry, EntryId};
use crate::matchmaker::MatchmakerResult::Matched;
use crate::matchmaker::{Matchmaker, MatchmakerResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct FlexibleMatchmaker {
    number_of_teams: i32,
    team_size: i32,
    max_entry_size: i32,
    min_entry_size: i32,
    #[serde(skip)]
    addends: Vec<Vec<i32>>,
    #[serde(skip)]
    entries: HashMap<EntryId, Entry>,
    #[serde(skip)]
    entries_by_size: HashMap<i32, Vec<EntryId>>,
}

impl FlexibleMatchmaker {
    pub fn new(
        number_of_teams: i32,
        team_size: i32,
        max_entry_size: i32,
        min_entry_size: i32,
    ) -> Self {
        let addends = find_unique_addends(team_size);
        Self {
            number_of_teams,
            team_size,
            addends,
            max_entry_size,
            min_entry_size,
            entries: HashMap::new(),
            entries_by_size: HashMap::new(),
        }
    }

    fn counter_from_slice(slice: &[i32]) -> HashMap<i32, i32> {
        let mut counter = HashMap::new();
        for &num in slice {
            *counter.entry(num).or_insert(0) += 1;
        }
        counter
    }

    fn use_team(composition: &[i32], available: &HashMap<i32, i32>) -> HashMap<i32, i32> {
        let mut new = available.clone();
        for &num in composition {
            if let Some(count) = new.get_mut(&num) {
                *count -= 1;
            }
        }
        new
    }

    fn can_form_team(composition: &[i32], available: &HashMap<i32, i32>) -> bool {
        let temp = Self::counter_from_slice(composition);
        for (&num, &count) in &temp {
            match available.get(&num) {
                Some(&available_count) if available_count >= count => continue,
                _ => return false,
            }
        }
        true
    }

    fn backtrack(
        chosen: &mut Vec<Vec<i32>>,
        available: &HashMap<i32, i32>,
        valid_compositions: &[Vec<i32>],
        num_teams: usize,
        results: &mut Vec<Vec<Vec<i32>>>,
    ) {
        if chosen.len() == num_teams {
            results.push(chosen.clone());
            return;
        }

        for comp in valid_compositions {
            if Self::can_form_team(comp, available) {
                let mut new_available = Self::use_team(comp, available);
                chosen.push(comp.clone());
                Self::backtrack(
                    chosen,
                    &mut new_available,
                    valid_compositions,
                    num_teams,
                    results,
                );
                chosen.pop();
            }
        }
    }

    pub fn build_teams(teams: &[i32], team_size: i32, num_teams: usize) -> Option<Vec<Vec<i32>>> {
        let teams_count = Self::counter_from_slice(teams);

        let addends: Vec<Vec<i32>> = find_unique_addends(team_size);

        let mut valid_compositions: Vec<Vec<i32>> = Vec::new();

        for addend in &addends {
            let addend_count = Self::counter_from_slice(addend);
            let valid = addend_count
                .iter()
                .all(|(&k, &v)| teams_count.get(&k).copied().unwrap_or(0) >= v);
            if valid {
                valid_compositions.push(addend.clone());
            }
        }

        let mut results: Vec<Vec<Vec<i32>>> = Vec::new();
        let available_counter = Self::counter_from_slice(teams);

        Self::backtrack(
            &mut vec![],
            &available_counter,
            &valid_compositions,
            num_teams,
            &mut results,
        );

        results.into_iter().next()
    }
}

impl Matchmaker for FlexibleMatchmaker {
    fn get_type_name(&self) -> String {
        String::from("flexible")
    }

    fn matchmake(&self) -> MatchmakerResult {
        let team_counts = Self::build_teams(
            self.get_entries()
                .iter()
                .map(|e| e.players.len() as i32)
                .collect::<Vec<i32>>()
                .as_slice(),
            self.team_size,
            self.number_of_teams as usize,
        );

        let Some(team_counts) = team_counts else {
            return MatchmakerResult::Skip(String::from(""));
        };

        let mut index_tracker: HashMap<i32, i32> = HashMap::new();
        let mut result_teams: Vec<Vec<EntryId>> = Vec::new();

        for sizes in team_counts {
            let mut team = Vec::new();
            for size in sizes {
                let index = *index_tracker.get(&size).unwrap_or(&0);
                index_tracker.insert(size, index + 1);
                let by_size = self.entries_by_size.get(&size);
                let Some(by_size) = by_size else {
                    return MatchmakerResult::Skip(String::from(""));
                };
                let picked = by_size.get(index as usize);
                let Some(picked) = picked else {
                    return MatchmakerResult::Skip(String::from(""));
                };
                team.push(picked.clone());
            }
            result_teams.push(team);
        }

        Matched(result_teams)
    }

    fn serialize(&self) -> Result<Value, Box<dyn Error>> {
        let json = serde_json::to_value(self)?;
        Ok(json)
    }

    fn remove_all(&mut self) -> Vec<Entry> {
        let entries: Vec<Entry> = self.entries.drain().map(|(_, entry)| entry).collect();
        self.entries_by_size.clear();
        entries
    }

    fn get_entries(&self) -> Vec<&Entry> {
        self.entries.values().collect()
    }

    fn remove_entry(&mut self, entry_id: &EntryId) -> Result<Entry, Box<dyn Error>> {
        let entry = self.entries.remove(entry_id);
        if let Some(entry) = entry {
            if let Some(teams) = self.entries_by_size.get_mut(&(entry.players.len() as i32)) {
                teams.retain(|&id| id != *entry_id);
            }
            Ok(entry)
        } else {
            Err("Entry not found".into())
        }
    }

    fn get_entry(&self, entry_id: &EntryId) -> Option<&Entry> {
        self.entries.get(entry_id)
    }

    fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn Error>> {
        let teams = self
            .entries_by_size
            .entry(entry.players.len() as i32)
            .or_insert_with(Vec::new);
        teams.push(entry.id);

        self.entries.insert(entry.id, entry);

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct BacktrackState {
    remaining: i32,
    current_combination: Vec<i32>,
    start: i32,
}

fn find_unique_addends(target: i32) -> Vec<Vec<i32>> {
    if target <= 0 {
        panic!("Target must be a positive integer");
    }

    let mut result: Vec<Vec<i32>> = Vec::new();
    let mut stack: VecDeque<BacktrackState> = VecDeque::new();
    stack.push_front(BacktrackState {
        remaining: target,
        current_combination: vec![],
        start: 1,
    });

    while let Some(state) = stack.pop_front() {
        let remaining = state.remaining;
        let current_combination = state.current_combination;
        let start = state.start;

        if remaining == 0 {
            result.push(current_combination);
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

    result
}

#[cfg(test)]
mod tests {
    use crate::entry::Entry;
    use crate::matchmaker::Matchmaker;
    use serde_json::Map;
    use uuid::Uuid;

    #[test]
    pub fn test() {
        let mut matchmaker = super::FlexibleMatchmaker::new(2, 1, 1, 1);

        matchmaker
            .add_entry(Entry::new(Uuid::new_v4(), vec![Uuid::new_v4()], Map::new()))
            .unwrap();
        matchmaker
            .add_entry(Entry::new(Uuid::new_v4(), vec![Uuid::new_v4()], Map::new()))
            .unwrap();

        let result = matchmaker.matchmake();

        println!("{:?}", result);
    }
}
