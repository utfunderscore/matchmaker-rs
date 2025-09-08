use axum::Json;
use axum::extract::ws::Message::Text;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use common::queue::{Entry, Queue, QueueResult};
use common::queue_tracker::QueueTracker;
use futures_util::SinkExt;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tracing::{debug, info};
use uuid::Uuid;
use crate::data::QueueData;

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
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    Path(queue): Path<String>,
) -> Response {
    ws.on_upgrade(|x| queue_join(x, queue_tracker, queue))
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct QueueJoinRequest {
    players: Vec<Uuid>,
}

pub async fn queue_join(
    socket: WebSocket,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    queue_name: String,
) {
    let (mut sender, mut receiver): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) =
        socket.split();

    let mut entry_ids: Vec<Uuid> = Vec::new();

    if queue_tracker.0.lock().await.get_queue(&queue_name).is_none() {
        let _ = sender
            .send(Text(
                format!("Queue '{queue_name}' does not exist").into(),
            ))
            .await;
        return;
    }

    let sender_mutex: Arc<Mutex<SplitSink<WebSocket, Message>>> = Arc::new(Mutex::new(sender));

    while let Some(Ok(Text(text))) = receiver.next().await {
        debug!("Received join request: {}", text);

        let join_request: Result<QueueJoinRequest, _> = serde_json::from_str(&text);
        if join_request.is_err() {
            let mut sender = sender_mutex.lock().await;

            let _ = &sender
                .send(Text("Invalid join request format".into()))
                .await;
            continue;
        }
        let join_result: Result<Uuid, String> = on_join_request(
            &queue_name,
            join_request.unwrap(),
            queue_tracker.0.clone(),
            sender_mutex.clone(),
        )
        .await
        .map_err(|e| e.to_string());

        match join_result {
            Ok(id) => {
                entry_ids.push(id);
            }
            Err(err) => {
                let mut sender = sender_mutex.lock().await;
                let _ = sender
                    .send(Text(format!("Failed to join queue: {err}").into()))
                    .await;
                info!("Failed to join queue for request:. Error: {}", text);
                continue;
            }
        }

        if let Ok(_) = join_result {
        } else {
            info!("Failed to join queue for request: {}", text);
            continue;
        }
    }

    // Socket closed here
    // Remove entries that have not found a match
    for id in entry_ids {
        let queue_tracker = queue_tracker.lock().await;
        let queue = queue_tracker.get_queue(&queue_name);
        if let Some(queue) = queue {
            let mut queue = queue.lock().await;
            let _ = queue.remove_entry(&id);
        } else {
            debug!("Queue '{}' not found for entry {}", queue_name, id);
        }
    }

    debug!("WebSocket connection closed for queue: {}", queue_name);
}

async fn on_join_request(
    queue_name: &String,
    join_request: QueueJoinRequest,
    queue_tracker: Arc<Mutex<QueueTracker>>,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
) -> Result<Uuid, Box<dyn Error>> {
    let queue_tracker = queue_tracker.lock().await;
    let queue = queue_tracker
        .get_queue(queue_name)
        .ok_or("Queue not found")?;

    let mut queue = queue.lock().await;
    let entry = Entry::new(join_request.players);
    let entry_id = entry.id();

    let receiver = queue.join_queue(entry)?;

    tokio::spawn(async move {
        let queue_result = receiver.await;
        if let Ok(queue_result) = queue_result {
            let mut sender = sender.lock().await;

            match queue_result {
                QueueResult::Success(teams, game) => {
                    let response = json!({
                        "status": "success",
                        "teams": teams,
                        "game": game,
                    });
                    let _ = sender.send(Text(response.to_string().into())).await;
                }
                QueueResult::Error(err) => {
                    let response = json!({
                        "status": "error",
                        "message": err,
                    });
                    let _ = sender.send(Text(response.to_string().into())).await;
                }
            }
        } else {
            debug!("Failed to receive queue result for entry {}", entry_id);
        }
    });

    Ok(entry_id)
    // queue tracker dropped
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
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::String(String::from("Failed to convert matchmaker to json"))))
        };
        let queue_data = QueueData::new(name.clone(), entries.into_iter().cloned().collect(), matchmaker);

        mapped.push(queue_data);
    }

    let Ok(queues) = serde_json::to_value(mapped) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(Value::String(String::from("Error occurred converting to json."))))
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
            let matchmaker_settings:  Result<Value, Box<dyn Error>> = queue.matchmaker().serialize();
            let Ok(matchmaker_settings) = matchmaker_settings else {
                return (StatusCode::INTERNAL_SERVER_ERROR, String::from("Error occurred converting to json."))
            };

            let queue_data = QueueData::new(name, queue.get_entries().into_iter().cloned().collect(), matchmaker_settings);
            let response = serde_json::to_string(&queue_data);
            match response {
                Ok(json) => {
                    (StatusCode::OK, json)}
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, String::from("Error occurred converting to json."))
                }
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            format!("Queue '{}' not found", name),
        ),
    }
}
