use crate::matchmaker::serializer::Registry;
use crate::queues::queue_pool::QueuePool;
use crate::queues::queue_ticker::QueueTicker;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait QueueStore {
    fn load(&self) -> Result<QueuePool, String>;

    async fn save(&self, queue_pool: Arc<Mutex<QueuePool>>) -> Result<(), String>;
}

pub struct FlatFile {
    path: String,
    serializer_registry: Registry,
}

impl FlatFile {
    pub fn new(path: String, serializer_registry: Registry) -> FlatFile {
        FlatFile {
            path,
            serializer_registry,
        }
    }

    fn get_read_file(&self) -> Result<File, String> {
        File::open(self.path.clone()).map_err(|x| x.to_string())
    }

    fn get_write_file(&self) -> Result<File, String> {
        File::create(self.path.clone()).map_err(|x| x.to_string())
    }
}

impl QueueStore for FlatFile {
    fn load(&self) -> Result<QueuePool, String> {
        if !Path::exists(self.path.clone().as_ref()) {
            let mut file = self.get_write_file()?;
            file.write_all("[]".as_bytes()).map_err(|x| x.to_string())?;
        }

        let mut queue_pool = QueuePool::new();
        let reader = BufReader::new(self.get_read_file()?);

        let json: Value = serde_json::from_reader(reader).map_err(|x| x.to_string())?;
        let json_array = json.as_array().ok_or("Not json array")?;

        for value in json_array {
            let matchmaker = QueueTicker::load(value, &self.serializer_registry)?;

            queue_pool.add_ticker(matchmaker)?;
        }

        Ok(queue_pool)
    }

    async fn save(&self, queue_pool: Arc<Mutex<QueuePool>>) -> Result<(), String> {
        let queue_pool = queue_pool.lock().await;

        let writer = BufWriter::new(self.get_write_file()?);

        let queues = queue_pool
            .queue_tickers
            .read()
            .map_err(|x| format!("{x:?}"))?;

        let mut serialized_queues: Vec<Value> = Vec::new();

        for x in queues.values() {
            let queue = x.lock().map_err(|x| format!("{x:?}"))?;

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
        serde_json::to_writer(writer, &serialized_queues).map_err(|x| format!("{x:?}"))?;
        Ok(())
    }
}
