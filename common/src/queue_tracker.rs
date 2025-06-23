use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker};
use crate::queue::Queue;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct QueueTracker {
    directory: PathBuf,
    queues: HashMap<String, Queue>,
}

impl QueueTracker {
    pub fn new<T>(path: T) -> Result<Self, Box<dyn Error>>
    where
        T: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create directory for queue tracker");
        }

        let mut queues: HashMap<String, Queue> = HashMap::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let queue_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid file name")?
                    .to_string();

                let queue = Queue::from(path.clone())?;
                queues.insert(queue_name, queue);
            }
        }

        Ok(QueueTracker {
            directory: path,
            queues: HashMap::new(),
        })
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

        
        queue.save(&name, &self.directory).map_err(|e| format!("Failed to save queue: {}", e))?;
        self.queues.insert(name, queue);

        Ok(())
    }

    pub fn get_queue(&self, name: &str) -> Option<&Queue> {
        self.queues.get(name)
    }

    pub fn get_queues(&self) -> &HashMap<String, Queue> {
        &self.queues
    }
}
