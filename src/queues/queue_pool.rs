use crate::game::game::Game;
use crate::matchmaker::serializer::SerializerRegistry;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_ticker::QueueTicker;
use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::oneshot::Receiver;
use uuid::Uuid;

pub struct QueuePool {
    pub queue_tickers: RwLock<HashMap<String, Arc<Mutex<QueueTicker>>>>,
    queue_creators: HashMap<
        String,
        Box<dyn Fn(String, Value) -> Result<Arc<Mutex<QueueTicker>>, String> + Send + Sync>,
    >,
}

impl QueuePool {
    pub fn new() -> Self {
        Self {
            queue_tickers: RwLock::new(HashMap::new()),
            queue_creators: HashMap::new(),
        }
    }

    pub fn add_ticker(&mut self, queue: Arc<Mutex<QueueTicker>>) -> Result<(), String> {
        let queues = self
            .queue_tickers
            .get_mut()
            .map_err(|x| format!("{:?}", x))?;

        let name = queue.lock().unwrap().get_queue().name.clone();
        queues.insert(name, queue);

        Ok(())
    }

    pub fn get_queue_copy(&self, queue_name: String) -> Result<Queue, String> {
        let queues = self.queue_tickers.read();
        if queues.is_err() {
            return Err(format!("Could not find queue with name {queue_name}"));
        }
        let queues = queues.unwrap();

        let queue_option = queues
            .get(&queue_name)
            .map(|x| x.lock().unwrap().get_queue().clone());

        queue_option.ok_or(format!("Could not find queue with name {queue_name}"))
    }

    pub fn join_queue(
        &self,
        queue_name: &String,
        queue_entry: QueueEntry,
    ) -> Result<Receiver<Result<Game, String>>, String> {
        let queues_lock = self.queue_tickers.write();
        if queues_lock.is_err() {
            return Err(format!("Could not find queue with name {queue_name}"));
        }

        let mut queues = queues_lock.unwrap();

        let queue_ticker = queues
            .get_mut(queue_name)
            .ok_or(format!("Queue with name {} not found", queue_name))?;

        let mut queue_ticker = queue_ticker.lock().map_err(|x| format!("{:?}", x))?;

        queue_ticker.add_team(queue_entry)
    }

    pub fn leave_queue(&self, queue_name: &String, entry_id: &Uuid) -> Result<(), String> {
        let mut queue_lock = self
            .queue_tickers
            .write()
            .map_err(|_x| "Failed to get lock access")?;
        let queue: &mut Arc<Mutex<QueueTicker>> = queue_lock
            .get_mut(queue_name)
            .ok_or_else(|| format!("Queue {} not found", queue_name))?;

        let mut queue_ticker = queue.lock().map_err(|x| format!("{}", x))?;

        queue_ticker.get_queue_mut().remove_team(&entry_id)?;

        Ok(())
    }

    ///Creates a copy of the current existing queues
    pub fn get_queues(&self) -> Result<Vec<Queue>, String> {
        let queue_tickers = self.queue_tickers.read().map_err(|x| format!("{}", x))?;

        let mut queues = Vec::<Queue>::new();

        Ok(queue_tickers
            .values()
            .map(|x| x.lock().unwrap().get_queue().clone())
            .collect())
    }

    pub fn add_creator(
        &mut self,
        creator_name: String,
        creator: Box<
            dyn Fn(String, Value) -> Result<Arc<Mutex<QueueTicker>>, String> + Send + Sync,
        >,
    ) {
        self.queue_creators.insert(creator_name, Box::new(creator));
    }

    pub fn create_queue(
        &mut self,
        queue_name: String,
        creator_name: String,
        data: Value,
    ) -> Result<Queue, String> {
        log::debug!(
            "Creating queue with name {} and type {}",
            queue_name,
            creator_name
        );

        let queue_creator = self
            .queue_creators
            .get(&creator_name)
            .ok_or("Could not find queue type with that id")?;

        let queue_ticker_mutex = queue_creator(queue_name, data)?;

        let queue = queue_ticker_mutex
            .clone()
            .lock()
            .map_err(|x| format!("{:?}", x))?
            .get_queue()
            .clone();

        self.add_ticker(queue_ticker_mutex)?;

        Ok(queue)
    }

    pub fn queue_exists(&self, queue_name: &String) -> bool {
        let queue_ticker = self.queue_tickers.read();

        queue_ticker.unwrap().contains_key(queue_name)
    }
}
