use crate::data::QueueData;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use common::queue::Queue;
use common::queue_tracker::QueueTracker;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

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
    let mut registry = registry.lock().await;

    if registry.get_queue(&request.name).is_some() {
        return Err(format!("Queue '{}' already exists", request.name).into());
    }

    registry.create_queue(request.name, request.matchmaker, request.settings)?;

    Ok(())
}

#[axum::debug_handler]
pub async fn get_queues_route(
    tracker: State<Arc<Mutex<QueueTracker>>>,
) -> (StatusCode, Json<Value>) {
    let tracker = tracker.lock().await;

    let queues: &HashMap<String, Arc<Mutex<Queue>>> = tracker.get_queues();
    let mut mapped: Vec<QueueData> = Vec::with_capacity(queues.len());

    for (name, queue_mutex) in queues {
        let queue = queue_mutex.lock().await;
        let entries = queue.get_entries();
        let matchmaker = queue.matchmaker().serialize();
        let Ok(matchmaker) = matchmaker else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Value::String(String::from(
                    "Failed to convert matchmaker to json",
                ))),
            );
        };
        let queue_data = QueueData::new(
            name.clone(),
            entries.into_iter().cloned().collect(),
            matchmaker,
        );

        mapped.push(queue_data);
    }

    let Ok(queues) = serde_json::to_value(mapped) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Value::String(String::from(
                "Error occurred converting to json.",
            ))),
        );
    };

    (StatusCode::OK, Json(queues))
}
#[axum::debug_handler]
pub async fn get_queue(
    registry: State<Arc<Mutex<QueueTracker>>>,
    Path(name): Path<String>,
) -> (StatusCode, String) {
    let registry = registry.lock().await;

    let queue = registry.get_queue(&name);

    match queue {
        Some(q) => {
            let queue: MutexGuard<Queue> = q.lock().await;
            let matchmaker_settings: Result<Value, Box<dyn Error>> = queue.matchmaker().serialize();
            let Ok(matchmaker_settings) = matchmaker_settings else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Error occurred converting to json."),
                );
            };

            let queue_data = QueueData::new(
                name,
                queue.get_entries().into_iter().cloned().collect(),
                matchmaker_settings,
            );
            let response = serde_json::to_string(&queue_data);
            match response {
                Ok(json) => (StatusCode::OK, json),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Error occurred converting to json."),
                ),
            }
        }
        None => (StatusCode::NOT_FOUND, format!("Queue '{}' not found", name)),
    }
}

#[axum::debug_handler]
pub async fn get_player_queue(
    registry: State<Arc<Mutex<QueueTracker>>>,
    Path(player): Path<String>,
) -> (StatusCode, String) {
    let tracker = registry.lock().await;

    let Ok(player) = Uuid::from_str(&player) else {
        return (StatusCode::BAD_REQUEST, String::from("Invalid player id"));
    };

    let Some(queue_name) = tracker.get_queue_by_player(&player).await else {
        return (StatusCode::NOT_FOUND, String::from("Invalid player id"));
    };

    let queue = tracker.get_queue(&queue_name);

    match queue {
        Some(q) => {
            let queue: MutexGuard<Queue> = q.lock().await;
            let matchmaker_settings: Result<Value, Box<dyn Error>> = queue.matchmaker().serialize();
            let Ok(matchmaker_settings) = matchmaker_settings else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Error occurred converting to json."),
                );
            };

            let queue_data = QueueData::new(
                queue_name,
                queue.get_entries().into_iter().cloned().collect(),
                matchmaker_settings,
            );
            let response = serde_json::to_string(&queue_data);
            match response {
                Ok(json) => (StatusCode::OK, json),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from("Error occurred converting to json."),
                ),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            format!("Queue '{}' not found", queue_name),
        ),
    }
}
