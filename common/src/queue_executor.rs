use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::task;
use crate::registry::Registry;

pub fn schedule(queue_name: String, registry: Arc<Mutex<Registry>>) {
    
    // task::spawn(async move {
    //     
    //     loop {
    //         let mut registry = registry.lock().await;
    //         if let Some(queue) = registry.get_queue(&queue_name) {
    //             queue.tick(&registry).unwrap();
    //         } else {
    //             break;
    //         }
    //     }
    // });
    
    todo!()
    
}