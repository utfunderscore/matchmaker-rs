use crate::codec::Codec;
use crate::matchmaker::Matchmaker;
use crate::queue::Queue;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

pub type ThreadMatchmaker = dyn Matchmaker + Send + Sync;

pub struct Registry {
    directory: PathBuf,
    codec: Codec,
    matchmakers: HashMap<String, Box<ThreadMatchmaker>>,
    queues: HashMap<String, Queue>,
}

impl Registry {
    pub fn new<P: Into<PathBuf>>(base_dir: P, codec: Codec) -> Result<Self, String> {
        let dir = base_dir.into();
        // Ensure the directory exists
        
        let mut matchmakers: HashMap<String, Box<ThreadMatchmaker>> = HashMap::new();
        let mut queues = HashMap::new();
        
        if !dir.exists() {
            std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            // iterate over existing files in the directory
            for entry in std::fs::read_dir(&dir).map_err(|e| format!("Failed to read directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
                    let queue_data: Value = serde_json::from_reader(file)
                        .map_err(|e| format!("Failed to parse queue data: {}", e))?;
                    
                    if let Some(matchmaker_name) = queue_data.get("matchmaker").and_then(Value::as_str) {
                        let matchmaker_deserializer = codec.get_deserializer(matchmaker_name)
                            .ok_or_else(|| format!("Matchmaker '{}' not found", matchmaker_name))?;
                        
                        let matchmaker = matchmaker_deserializer(queue_data.get("settings").cloned().unwrap_or_default())
                            .map_err(|e| format!("Failed to deserialize matchmaker: {}", e))?;
                        
                        matchmakers.insert(matchmaker_name.to_string(), matchmaker);
                        
                        let queue = Queue::new(matchmaker_name.to_string());
                        queues.insert(path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase(), queue);
                    }
                }
            }
        }

        Ok(Registry {
            directory: dir,
            codec,
            matchmakers,
            queues,
        })
    }

    pub fn get_queue(&mut self, name: &str) -> Option<&mut Queue> {
        self.queues.get_mut(&name.to_lowercase())
    }

    pub fn get_queues(&self) -> Vec<Queue> {
        self.queues.values().cloned().collect()
    }

    pub fn register_queue(
        &mut self,
        name: &str,
        matchmaker_name: &str,
        settings: Value,
    ) -> Result<(), String> {
        // Check if the matchmaker already exists
        if self.codec.get_deserializer(&matchmaker_name).is_none() {
            return Err("Matchmaker does not exist".to_string());
        }

        // Check if the queue already exists
        if self.get_queue(&name).is_some() {
            return Err("Queue already exists".to_string());
        }

        // Check if a matchmaker constructor is registered for the specified matchmaker
        let constructor = self
            .codec
            .get_deserializer(&matchmaker_name)
            .ok_or("Invalid matchmaker specified")?;

        // Create the matchmaker using the constructor
        let matchmaker: Box<ThreadMatchmaker> = constructor(settings)?;

        // Register the matchmaker and queue in the registry
        self.register_matchmaker(&matchmaker_name, matchmaker);

        let queue = Queue::new(String::from(matchmaker_name));

        self.save_queue(name, &queue.clone())?;
        self.queues.insert(name.to_lowercase(), queue);
        Ok(())
    }

    pub fn register_matchmaker(&mut self, name: &str, matchmaker: Box<ThreadMatchmaker>) {
        self.matchmakers.insert(name.to_lowercase(), matchmaker);
    }

    pub fn get_matchmaker(&self, name: &str) -> Option<&Box<ThreadMatchmaker>> {
        self.matchmakers.get(&name.to_lowercase())
    }

    pub fn save_queue(&self, name: &str, queue: &Queue) -> Result<(), String> {
        let matchmaker = self
            .get_matchmaker(&queue.matchmaker)
            .ok_or(String::from("Could not find matchmaker"))?;
        let matchmaker_data = matchmaker.serialize()?;

        let queue_data = json!({
            "matchmaker": queue.matchmaker,
            "settings": matchmaker_data,
        });

        let mut path = self.directory.clone();
        path.push(format!("{}.json", name));

        let file = File::create(&path).map_err(|e| format!("Failed to create file: {}", e))?;

        serde_json::to_writer_pretty(file, &queue_data)
            .map_err(|e| format!("Failed to save queue: {}", e))?;

        Ok(())
    }
}
