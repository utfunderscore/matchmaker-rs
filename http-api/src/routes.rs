use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use common::queue::Queue;
use common::registry::{Registry, ThreadMatchmaker};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
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
    registry: State<Arc<Mutex<Registry>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    match create_queue(registry.0, request.0).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json::from(json!({"message": "Queue created successfully"})),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json::from(json!({"error": e}))),
    }
}

pub async fn create_queue(
    registry: Arc<Mutex<Registry>>,
    request: CreateQueueRequest,
) -> Result<(), String> {
    let mut registry = registry.lock().await;

    // Create and register the queue
    registry.register_queue(&request.name, &request.matchmaker, request.settings)?;

    Ok(())
}

#[axum::debug_handler]
pub async fn get_queues_route(
    registry: State<Arc<Mutex<Registry>>>,
) -> (StatusCode, Json<Vec<Queue>>) {
    let registry = registry.0.lock().await;
    (StatusCode::OK, Json::from(registry.get_queues()))
}
#[axum::debug_handler]
pub async fn get_queue(
    registry: State<Arc<Mutex<Registry>>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let registry = registry.0.lock().await;
    match registry.get_queue(&name) {
        None => (StatusCode::NOT_FOUND, Json::from(json!({"error": "test"}))),
        Some(queue) => {
            let value = serde_json::to_value(queue).unwrap();
            (StatusCode::OK, Json::from(value))
        }
    }
}
