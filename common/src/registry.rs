use crate::matchmaker::Matchmaker;
use crate::queue::Queue;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub type ThreadMatchmaker = dyn Matchmaker + Send + Sync;

pub type MatchmakerConstructor = fn(&Map<String, Value>) -> Result<Box<ThreadMatchmaker>, String>;

pub struct Registry {
    matchmakers: HashMap<String, Box<ThreadMatchmaker>>,
    creators: HashMap<String, MatchmakerConstructor>,
    queues: HashMap<String, Queue>, // Placeholder for queue management
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            matchmakers: HashMap::new(),
            creators: HashMap::new(),
            queues: HashMap::new(),
        }
    }

    pub fn get_queue(&self, name: &str) -> Option<&Queue> {
        self.queues.get(&name.to_lowercase())
    }

    pub fn register_queue(&mut self, name: &str, queue: Queue) {
        self.queues.insert(name.to_lowercase(), queue);
    }

    pub fn register_matchmaker(&mut self, name: &str, matchmaker: Box<ThreadMatchmaker>) {
        self.matchmakers.insert(name.to_lowercase(), matchmaker);
    }

    pub fn get_matchmaker(&self, name: &str) -> Option<&Box<ThreadMatchmaker>> {
        self.matchmakers.get(&name.to_lowercase())
    }

    pub fn register_constructor(&mut self, name: &str, constructor: MatchmakerConstructor) {
        self.creators.insert(name.to_lowercase(), constructor);
    }

    pub fn get_constructor(&self, name: &str) -> Option<&MatchmakerConstructor> {
        self.creators.get(&name.to_lowercase())
    }
}
