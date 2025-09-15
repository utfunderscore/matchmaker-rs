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

/// Creates a new queue.
///
/// **Request:**
/// - Method: `POST`
/// - Path: `/queues`
/// - Body: JSON object with fields:
///   - `name` (String): Name of the queue.
///   - `matchmaker` (String): Matchmaker type.
///   - `settings` (serde_json::Value): Matchmaker settings.
/// - Example:
///   {
///     "name": "queue1",
///     "matchmaker": "default",
///     "settings": { ... }
///   }
///
/// **Response:**
/// - `201 Created`: Queue created successfully.
///   - Body: `{ "status": "Queue created successfully" }`
/// - `400 Bad Request`: Error creating queue.
///   - Body: Error message string.
#[axum::debug_handler]
pub async fn create_queue_route(
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    request: Json<CreateQueueRequest>,
) -> (StatusCode, Json<Value>) {
    let queue_tracker = queue_tracker.0;

    match QueueTracker::create(
        queue_tracker,
        request.name.clone(),
        request.matchmaker.clone(),
        request.settings.clone(),
        true,
    )
    .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({"status": "Queue created successfully"})),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(Value::String(e.to_string()))),
    }
}

/// Lists all queue names.
///
/// **Request:**
/// - Method: `GET`
/// - Path: `/queues`
///
/// **Response:**
/// - `200 OK`: Returns a JSON array of queue names.
///   - Body: `[ "queue1", "queue2", ... ]`
/// - `500 Internal Server Error`: Error serializing queue list.
///   - Body: `{ "error": "..." }`
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

/// Gets detailed information about a specific queue.
///
/// **Request:**
/// - Method: `GET`
/// - Path: `/queues/{name}`
/// - Path parameter: `name` (String): Name of the queue.
///
/// **Response:**
/// - `200 OK`: Returns queue data as JSON string.
///   - Body: JSON string with queue details (see `QueueData` struct).
/// - `404 Not Found`: Queue not found.
///   - Body: `"Queue not found"`
/// - `500 Internal Server Error`: Error serializing queue data.
///   - Body: `"Error occurred converting to json."`
#[axum::debug_handler]
pub async fn get_queue(
    registry: State<Arc<Mutex<QueueTracker>>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<Value>) {
    let registry = registry.lock().await;

    let queue: Option<Arc<Mutex<Queue>>> = registry.get_queue(&name).await;

    let Some(queue) = queue else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Queue {} does not exist", name)})),
        );
    };

    let queue = queue.lock().await;
    let matchmaker = queue.matchmaker();

    let Ok(matchmaker_settings) = matchmaker.serialize() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            // String::from("Error occurred converting to json."),
            Json(json!({"error": "Error occurred converting to json."})),
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

    let queue_data_json = serde_json::to_value(&queue_data);
    let Ok(queue_data) = queue_data_json else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Error occurred converting to json."})),
        );
    };

    (StatusCode::OK, Json(queue_data))
}
