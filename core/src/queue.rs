use crate::matchmaker;
use crate::queue_entry::QueueEntry;
use matchmaker::Matchmaker;
use std::collections::HashMap;
use std::hash::Hash;
use uuid::Uuid;

pub struct Queue<T> {
    name: String,
    teams: HashMap<Uuid, T>,
    matchmaker: Box<dyn Matchmaker<T>>,
}

impl<T> Queue<T>
where
    T: QueueEntry + Hash + Eq + Clone,
{
    pub fn new(name: String, matchmaker: Box<dyn Matchmaker<T>>) -> Self {
        Queue {
            name,
            teams: HashMap::new(),
            matchmaker,
        }
    }

    pub fn add_team(&mut self, team: T) -> Result<(), String> {
        if self.teams.contains_key(&team.id()) {
            return Err("Team already exists in the queue".to_string());
        }
        self.teams.insert(team.id(), team);
        Ok(())
    }
    
    pub fn remove_team(&mut self, team: &T) -> Option<T> {
        self.teams.remove(&team.id())
    }
    
    pub fn matchmake(&mut self) -> Result<Vec<Vec<T>>, String> {
        let teams: Vec<T> = self.teams.values().into_iter().cloned().collect();

        let teams = self.matchmaker.matchmake(&teams)?;

        let teams = teams
            .into_iter()
            .map(|team_ids| {
                team_ids
                    .into_iter()
                    .filter_map(|id| self.teams.remove(&id))
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<Vec<T>>>();
        
        Ok(teams)
    }
}
