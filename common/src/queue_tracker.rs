use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker};
use crate::queue::{Queue, QueueTrait};
use serde_json::Value;
use std::collections::HashMap;

pub struct QueueTracker {
    queues: HashMap<String, Box<dyn QueueTrait>>,
}

impl Default for QueueTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueTracker {
    pub fn new() -> Self {
        QueueTracker {
            queues: HashMap::new(),
        }
    }

    pub fn create_queue(
        &mut self,
        name: String,
        matchmaker: String,
        settings: Value,
    ) -> Result<(), String> {
        let deserializer: &Deserializer =
            matchmaker::get_deserializer(&matchmaker).ok_or(format!("Unknown matchmaker: {}", matchmaker))?;
        let matchmaker: Box<dyn Matchmaker + Send + Sync> = deserializer(settings)
            .map_err(|e| format!("Failed to deserialize matchmaker: {}", e))?;
        let queue = Queue::new(matchmaker);

        self.queues.insert(name, Box::new(queue));

        Ok(())
    }

    pub fn get_queue(&self, name: &str) -> Option<&dyn QueueTrait> {
        self.queues.get(name).map(|v| &**v)
    }

    pub fn get_queues(&self) -> &HashMap<String, Box<dyn QueueTrait>> {
        &self.queues
    }
}
