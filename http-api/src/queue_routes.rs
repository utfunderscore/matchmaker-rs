use crate::data::QueueData;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use common::queue::Queue;
use common::queue_tracker::QueueTracker;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct CreateQueueRequest {
    name: String,
    matchmaker: String,
    settings: Value,
}

#[axum::debug_handler]
pub async fn create_queue_route(
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    let queue_tracker = queue_tracker.0;

    match create_queue(queue_tracker, request.0).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({"status": "Queue created successfully"})),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(Value::String(e.to_string()))),
    }
}

pub async fn create_queue(
    registry: Arc<Mutex<QueueTracker>>,
    request: CreateQueueRequest,
) -> Result<(), Box<dyn Error>> {
    QueueTracker::create(registry, request.name, request.matchmaker, request.settings).await?;
    Ok(())
}

#[axum::debug_handler]
pub async fn get_queues_route(
    tracker: State<Arc<Mutex<QueueTracker>>>,
) -> (StatusCode, Json<Value>) {
    let tracker = tracker.lock().await;

    let queues: Vec<&String> = tracker.get_queues().keys().collect();
    let json = serde_json::to_value(&queues);

    match json {
        Ok(json) => (StatusCode::OK, Json(json)),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}
#[axum::debug_handler]
pub async fn get_queue(
    registry: State<Arc<Mutex<QueueTracker>>>,
    Path(name): Path<String>,
) -> (StatusCode, String) {
    let registry = registry.lock().await;

    let queue: Option<Arc<Mutex<Queue>>> = registry.get_queue(&name).await;

    let Some(queue) = queue else {
        return (StatusCode::NOT_FOUND, String::from("Queue not found"));
    };

    let queue = queue.lock().await;
    let matchmaker = queue.matchmaker();

    let Ok(matchmaker_settings) = matchmaker.serialize() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Error occurred converting to json."),
        );
    };

    let matchmaker = json!({
        "type": matchmaker.get_type_name(),
        "settings": matchmaker_settings
    });

    let queue_data = QueueData::new(
        name,
        queue.entries().values().cloned().collect(),
        matchmaker,
    );

    let queue_data_json = serde_json::to_string(&queue_data);
    let Ok(queue_data) = queue_data_json else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("Error occurred converting to json."),
        );
    };

    (StatusCode::OK, queue_data)
}
