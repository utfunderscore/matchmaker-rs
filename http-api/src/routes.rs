use std::collections::HashMap;
use std::error::Error;
use crate::data::QueueData;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use common::queue_tracker::QueueTracker;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use common::queue::{Queue};

#[derive(Serialize, Deserialize)]
pub struct CreateQueueRequest {
    name: String,
    matchmaker: String,
    settings: Value,
}

#[axum::debug_handler]
pub async fn create_queue_route(
    registry: State<Arc<Mutex<QueueTracker>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    match create_queue(registry.0.clone(), request.0).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({"status": "Queue created successfully"}))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(Value::String(e.to_string()))),
    }
}

pub async fn create_queue(
    registry: Arc<Mutex<QueueTracker>>,
    request: CreateQueueRequest,
) -> Result<(), Box<dyn Error>> {
    let mut registry = registry.lock().await;

    registry.create_queue(
        request.name,
        request.matchmaker,
        request.settings,
    )?;

    Ok(())
}

#[axum::debug_handler]
pub async fn get_queues_route(
    registry: State<Arc<Mutex<QueueTracker>>>,
) -> (StatusCode, Json<Vec<QueueData>>) {
    let tracker = registry.lock().await;

    let queues: &HashMap<String, Queue> = tracker.get_queues();
    let queue_data: Vec<QueueData> = queues.iter().map(|(name, queue)| {
        QueueData {
            name: name.clone(),
            entries: queue.get_entries(), // Assuming get_entries() returns Vec<QueueEntry>
        }
    }).collect();

    (StatusCode::OK, Json(queue_data))
}
#[axum::debug_handler]
pub async fn get_queue(
    registry: State<Arc<Mutex<QueueTracker>>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let registry = registry.lock().await;

    let queue = registry.get_queue(&name);

    match queue {
        Some(q) => {
            let entries = q.get_entries(); // Assuming get_entries() returns Vec<QueueEntry>
            let response = json!({
                "name": name,
                "entries": entries,
            });
            (StatusCode::OK, Json(response))
        },
        None => (StatusCode::NOT_FOUND, Json(Value::String(format!("Queue '{}' not found", name)))),
    }
}
