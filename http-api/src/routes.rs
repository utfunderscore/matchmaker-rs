use crate::AppData;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use common::queue::Queue;
use common::registry::ThreadMatchmaker;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    app_data: State<Arc<Mutex<AppData>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    match create_queue(app_data.0, request.0).await {
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
    app_data: Arc<Mutex<AppData>>,
    request: CreateQueueRequest,
) -> Result<(), String> {
    let mut app_data = app_data.lock().await;

    // Check if the matchmaker already exists
    if app_data
        .codec
        .get_deserializer(&request.matchmaker)
        .is_none()
    {
        return Err("Matchmaker does not exist".to_string());
    }

    // Check if the queue already exists
    if app_data.registry.get_queue(&request.name).is_some() {
        return Err("Queue already exists".to_string());
    }

    // Check if a matchmaker constructor is registered for the specified matchmaker
    let constructor = app_data
        .codec
        .get_deserializer(&request.matchmaker)
        .ok_or("Invalid matchmaker specified")?;

    // Create the matchmaker using the constructor
    let matchmaker: Box<ThreadMatchmaker> = constructor(request.settings)?;

    // Register the matchmaker and queue in the registry
    app_data
        .registry
        .register_matchmaker(&request.matchmaker, matchmaker);

    // Create and register the queue
    app_data
        .registry
        .register_queue(&request.name, Queue::new(request.name.clone()));

    Ok(())
}
