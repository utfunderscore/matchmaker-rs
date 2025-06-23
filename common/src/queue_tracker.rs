use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker};
use crate::queue::{Queue, QueueTrait};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct QueueTracker {
    directory: PathBuf,
    queues: HashMap<String, Box<dyn QueueTrait>>,
}

impl QueueTracker {
    pub fn new<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        QueueTracker {
            directory: path.into(),
            queues: HashMap::new(),
        }
    }

    pub fn create_queue(
        &mut self,
        name: String,
        matchmaker: String,
        settings: Value,
    ) -> Result<(), String> {
        let deserializer: &Deserializer = matchmaker::get_deserializer(&matchmaker)
            .ok_or(format!("Unknown matchmaker: {}", matchmaker))?;
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

    pub fn save(&self, name: &str, queue: Box<dyn QueueTrait>) -> Result<(), String> {
        let queue_json = queue.serialize()?;
        let file_path = self.directory.join(format!("{}.json", name));
        
        std::fs::write(&file_path, serde_json::to_string(&queue_json).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        
        Ok(())
    }
}
