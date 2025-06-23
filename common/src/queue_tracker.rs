use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker};
use crate::queue::{Queue};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct QueueTracker {
    directory: PathBuf,
    queues: HashMap<String, Box<Queue>>,
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
        let queue = Box::new(Queue::new(matchmaker));
        
        self.save(&name, &queue)?;
        self.queues.insert(name, queue);

        Ok(())
    }

    pub fn get_queue(&self, name: &str) -> Option<&Queue> {
        self.queues.get(name).map(|v| &**v)
    }

    pub fn get_queues(&self) -> &HashMap<String, Box<Queue>> {
        &self.queues
    }

    pub fn save(&self, name: &str, queue: &Queue) -> Result<(), String> {
        let queue_json = queue.serialize()?;

        let file_path = self.directory.join(format!("{}.json", name));
        std::fs::write(&file_path, serde_json::to_string(&queue_json).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Failed to write queue to file: {}", e))?;

        Ok(())
    }
}
