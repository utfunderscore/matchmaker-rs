use crate::codec::Codec;
use crate::matchmaker::Matchmaker;
use crate::queue::Queue;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

pub type ThreadMatchmaker = dyn Matchmaker + Send + Sync;

pub struct Registry {
    directory: PathBuf,
    codec: Codec,
    matchmakers: HashMap<String, Box<ThreadMatchmaker>>,
    queues: HashMap<String, Queue>,
}

impl Registry {
    pub fn new<P: Into<PathBuf>>(base_dir: P, codec: Codec) -> Self {
        Registry {
            directory: base_dir.into(),
            codec,
            matchmakers: HashMap::new(),
            queues: HashMap::new(),
        }
    }

    pub fn get_queue(&self, name: &str) -> Option<&Queue> {
        self.queues.get(&name.to_lowercase())
    }

    pub fn get_queues(&self) -> Vec<Queue> {
        self.queues.values().cloned().collect()
    }

    pub fn register_queue(&mut self, name: &str, matchmaker_name: &str, settings: Value) -> Result<(), String> {

        // Check if the matchmaker already exists
        if self.codec
            .get_deserializer(&matchmaker_name)
            .is_none()
        {
            return Err("Matchmaker does not exist".to_string());
        }

        // Check if the queue already exists
        if self.get_queue(&name).is_some() {
            return Err("Queue already exists".to_string());
        }

        // Check if a matchmaker constructor is registered for the specified matchmaker
        let constructor = self.codec
            .get_deserializer(&matchmaker_name)
            .ok_or("Invalid matchmaker specified")?;

        // Create the matchmaker using the constructor
        let matchmaker: Box<ThreadMatchmaker> = constructor(settings)?;

        // Register the matchmaker and queue in the registry
        self.register_matchmaker(&matchmaker_name, matchmaker);
        
        let queue = Queue::new(String::from(matchmaker_name));
        
        self.save_queue(name, &queue.clone(), &self.codec)?;
        self.queues.insert(name.to_lowercase(), queue);
        Ok(())
    }

    pub fn register_matchmaker(&mut self, name: &str, matchmaker: Box<ThreadMatchmaker>) {
        self.matchmakers.insert(name.to_lowercase(), matchmaker);
    }

    pub fn get_matchmaker(&self, name: &str) -> Option<&Box<ThreadMatchmaker>> {
        self.matchmakers.get(&name.to_lowercase())
    }

    pub fn save_queue(&self, name: &str, queue: &Queue, codec: &Codec) -> Result<(), String> {
        let matchmaker = self.get_matchmaker(&queue.matchmaker).ok_or(String::from("Could not find matchmaker"))?;
        let serializer = codec.get_serializer(&queue.matchmaker).ok_or(String::from("Could not find serializer"))?;

        let matchmaker_data = serializer(matchmaker).map_err(|e| format!("Failed to serialize matchmaker: {}", e))?;

        self.directory.clone().push(format!("{}.json", name));
        serde_json::to_writer_pretty(File::create(self.directory.clone()).unwrap(), &matchmaker_data).map_err(|e| format!("Failed to save queue: {}", e))?;
        Ok(())
    }
}
