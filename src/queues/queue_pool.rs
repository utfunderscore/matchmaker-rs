use crate::game::providers;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_ticker::QueueTicker;
use log::error;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::oneshot::Receiver;
use uuid::Uuid;

type QueueCreatorFunc =
    Box<dyn Fn(String, &Value) -> Result<Arc<Mutex<QueueTicker>>, String> + Send + Sync>;

pub struct QueuePool {
    pub queue_tickers: RwLock<HashMap<String, Arc<Mutex<QueueTicker>>>>,
    queue_creators: HashMap<String, QueueCreatorFunc>,
}

impl QueuePool {
    pub fn new() -> Self {
        Self {
            queue_tickers: RwLock::new(HashMap::new()),
            queue_creators: HashMap::new(),
        }
    }

    pub fn add_ticker(&mut self, queue: Arc<Mutex<QueueTicker>>) -> Result<(), String> {
        let queues = self.queue_tickers.get_mut().map_err(|x| format!("{x:?}"))?;

        let name = queue.lock().unwrap().get_queue().name.clone();
        queues.insert(name, queue);

        Ok(())
    }

    pub fn join_queue(
        &self,
        queue_name: &String,
        queue_entry: QueueEntry,
    ) -> Result<Receiver<Result<providers::Game, String>>, String> {
        let queues_lock = self.queue_tickers.write();
        if queues_lock.is_err() {
            error!("Could not find queue with name {}", queue_name);
            return Err(format!("Could not find queue with name {queue_name}"));
        }

        let mut queues = queues_lock.unwrap();

        let queue_ticker = queues
            .get_mut(queue_name)
            .ok_or(format!("Queue with name {queue_name} not found"))?;

        let mut queue_ticker = queue_ticker.lock().map_err(|x| format!("{x:?}"))?;

        queue_ticker.add_team(queue_entry)
    }

    pub fn leave_queue(&self, queue_name: &String, entry_id: &Uuid) -> Result<(), String> {
        let mut queue_lock = self
            .queue_tickers
            .write()
            .map_err(|_x| "Failed to get lock access")?;
        let queue: &mut Arc<Mutex<QueueTicker>> = queue_lock
            .get_mut(queue_name)
            .ok_or_else(|| format!("Queue {queue_name} not found"))?;

        let mut queue_ticker = queue.lock().map_err(|x| format!("{x}"))?;

        queue_ticker.get_queue_mut().remove_team(entry_id)?;

        Ok(())
    }

    ///Creates a copy of the current existing queues
    pub fn get_queues(&self) -> Result<Vec<Queue>, String> {
        let queue_tickers = self.queue_tickers.read().map_err(|x| format!("{x}"))?;

        Ok(queue_tickers
            .values()
            .map(|x| x.lock().unwrap().get_queue().clone())
            .collect())
    }

    pub fn add_creator(&mut self, creator_name: String, creator: QueueCreatorFunc) {
        self.queue_creators.insert(creator_name, Box::new(creator));
    }

    pub fn create_queue(
        &mut self,
        queue_name: String,
        creator_name: &str,
        data: &Value,
    ) -> Result<Queue, String> {
        log::debug!(
            "Creating queue with name {} and type {}",
            queue_name,
            creator_name
        );

        let queue_creator = self
            .queue_creators
            .get(creator_name)
            .ok_or("Could not find queue type with that id")?;

        let queue_ticker_mutex = queue_creator(queue_name, data)?;

        let queue = queue_ticker_mutex
            .clone()
            .lock()
            .map_err(|x| format!("{x:?}"))?
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
