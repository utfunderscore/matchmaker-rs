use crate::matchmaker;
use crate::matchmaker::{Deserializer, Matchmaker};
use crate::queue::Queue;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

pub struct QueueTracker {
    directory: PathBuf,
    queues: HashMap<String, Arc<Mutex<Queue>>>,
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

        let mut queues: HashMap<String, Arc<Mutex<Queue>>> = HashMap::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let queue_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Invalid file name")?
                    .to_string();

                let queue = Queue::from(path)?;
                queues.insert(queue_name, Arc::new(Mutex::new(queue)));
            }
        }

        let mut tracker = QueueTracker {
            directory: path,
            queues: HashMap::new(),
        };

        for (name, queue) in queues {
            tracker.register_queue(name, queue)?;
        }

        Ok(tracker)
    }

    pub fn create_queue(
        &mut self,
        name: String,
        matchmaker: String,
        settings: Value,
    ) -> Result<(), Box<dyn Error>> {
        let deserializer: &Deserializer = matchmaker::get_deserializer(&matchmaker)
            .ok_or(format!("Unknown matchmaker: {}", matchmaker))?;
        let matchmaker: Box<dyn Matchmaker + Send + Sync> = deserializer(settings)?;
        let queue = Queue::new(matchmaker);

        queue.save(&name, &self.directory)?;

        let queue_mutex = Arc::new(Mutex::new(queue));

        self.register_queue(name, queue_mutex)?;

        Ok(())
    }

    pub fn register_queue(
        &mut self,
        name: String,
        queue: Arc<Mutex<Queue>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.queues.contains_key(&name) {
            return Err(format!("Queue '{}' already exists", name).into());
        }
        let task_queue = Arc::clone(&queue);

        tokio::spawn(async move {
            loop {
                
                let mut queue = task_queue.lock().await;
                let result = queue.tick().await;
                
                drop(queue);
                
                if !result {
                    sleep(Duration::from_millis(50)).await;
                }
            }
        });

        self.queues.insert(name, queue);
        Ok(())
    }

    pub fn get_queue(&self, name: &str) -> Option<Arc<Mutex<Queue>>> {
        self.queues.get(name).cloned()
    }

    pub fn get_queues(&self) -> &HashMap<String, Arc<Mutex<Queue>>> {
        &self.queues
    }
}
