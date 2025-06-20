use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use common::queue::Queue;
use common::registry::{MatchmakerConstructor, Registry, ThreadMatchmaker};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct CreateQueueRequest {
    name: String,
    matchmaker: String,
    settings: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn create_queue_route(
    tracker: State<Arc<Mutex<Registry>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    match create_queue(tracker.0, request.0).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json::from(serde_json::json!({"message": "Queue created successfully"})),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json::from(serde_json::json!({"error": e})),
        ),
    }
}

pub async fn create_queue(
    registry: Arc<Mutex<Registry>>,
    request: CreateQueueRequest,
) -> Result<(), String> {
    let mut registry = registry.lock().unwrap();

    // Check if the matchmaker already exists
    if registry.get_matchmaker(&request.matchmaker).is_none() {
        return Err("Matchmaker does not exist".to_string());
    }

    // Check if the queue already exists
    if registry.get_queue(&request.name).is_none() {
        return Err("Queue already exists".to_string());
    }

    // Check if a matchmaker constructor is registered for the specified matchmaker
    let constructor = registry
        .get_constructor(&request.matchmaker)
        .ok_or("Invalid matchmaker specified")?;

    // Create the matchmaker using the constructor
    let matchmaker: Box<ThreadMatchmaker> = constructor(&request.settings.clone())?;

    // Register the matchmaker and queue in the registry
    registry.register_matchmaker(&request.matchmaker, matchmaker);

    // Create and register the queue
    registry.register_queue(&request.name, Queue::new(request.name.clone()));

    Ok(())
}
