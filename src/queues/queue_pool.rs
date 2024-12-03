use crate::game::game::Game;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_ticker::QueueTicker;
use rocket::tokio::sync::oneshot::{channel, Receiver};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub struct QueuePool {
    queue_tickers: RwLock<HashMap<String, Arc<Mutex<QueueTicker>>>>,
}

impl QueuePool {
    pub fn new() -> Self {
        Self {
            queue_tickers: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_ticker(&mut self, queue: Arc<Mutex<QueueTicker>>) {
        let queues = self
            .queue_tickers
            .get_mut()
            .expect("Unable to get write access");

        let name = queue.lock().unwrap().get_queue().name.clone();
        queues.insert(name, queue);
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
        queue_name: String,
        queue_entry: QueueEntry,
    ) -> Result<Receiver<Result<Game, String>>, String> {
        let queues = self.queue_tickers.write();
        if queues.is_err() {
            return Err(format!("Could not find queue with name {queue_name}"));
        }
        let mut queues = queues.unwrap();

        let queue = queues.get_mut(&queue_name);
        if queue.is_none() {
            return Err(format!("Could not find queue with name {queue_name}"));
        }
        let mut queue_ticker = queue.unwrap().lock().unwrap();

        let (sender, receiver) = channel::<Result<Game, String>>();

        queue_ticker.add_channel(queue_entry.id, sender);

        queue_ticker.get_queue_mut().add_team(queue_entry).clone()?;

        Ok(receiver)
    }

    ///Creates a copy of the current existing queues
    pub fn get_queues(&self) -> Vec<Queue> {
        let queue_tickers = self
            .queue_tickers
            .read()
            .expect("Unable to get read access");

        let mut queues = Vec::<Queue>::new();

        queue_tickers
            .values()
            .map(|x| x.lock().unwrap().get_queue().clone())
            .collect()
    }

    pub fn queue_exists(&self, queue_name: &String) -> bool {
        let queue_ticker = self.queue_tickers.read();

        queue_ticker.unwrap().contains_key(queue_name)
    }
}
