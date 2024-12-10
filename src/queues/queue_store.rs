use crate::matchmaker::serializer::SerializerRegistry;
use crate::queues::queue_pool::QueuePool;
use crate::queues::queue_ticker::QueueTicker;
use actix_web::web::Data;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::{Arc, Mutex};

pub trait QueueStore {
    fn load(&self) -> Result<QueuePool, String>;

    fn save(&self, queue_pool: Data<Arc<Mutex<QueuePool>>>) -> Result<(), String>;
}

pub struct FlatFileQueueStore {
    file: File,
    serializer_registry: SerializerRegistry,
}

impl FlatFileQueueStore {
    pub fn new(file: File, serializer_registry: SerializerRegistry) -> FlatFileQueueStore {
        FlatFileQueueStore {
            file,
            serializer_registry,
        }
    }
}

impl QueueStore for FlatFileQueueStore {
    fn load(&self) -> Result<QueuePool, String> {
        let mut queue_pool = QueuePool::new();
        let reader = BufReader::new(&self.file);

        let json: Value = serde_json::from_reader(reader).map_err(|x| x.to_string())?;
        let json_array = json.as_array().ok_or("Not json array")?;

        for value in json_array {
            let matchmaker = QueueTicker::load(value, &self.serializer_registry)?;

            queue_pool.add_ticker(matchmaker)?
        }

        Ok(queue_pool)
    }

    fn save(&self, queue_pool: Data<Arc<Mutex<QueuePool>>>) -> Result<(), String> {
        let queue_pool = queue_pool.lock().map_err(|x| x.to_string())?;

        let writer = BufWriter::new(&self.file);

        let queues = queue_pool
            .queue_tickers
            .read()
            .map_err(|x| format!("{:?}", x))?;

        let mut serialized_queues: Vec<Value> = Vec::new();

        for x in queues.values() {
            let queue = x.lock().map_err(|x| format!("{:?}", x))?;

            let save_result = queue.save(&self.serializer_registry);
            if save_result.is_err() {
                println!(
                    "Failed to save queue '{}': {:?}",
                    queue.get_queue().name,
                    save_result.err().unwrap()
                );
                continue;
            }
            serialized_queues.push(save_result?);
        }

        println!("{:?}", serialized_queues);

        serde_json::to_writer(writer, &serialized_queues).map_err(|x| format!("{:?}", x))?;

        Ok(())
    }
}
