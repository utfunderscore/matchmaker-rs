use axum::Json;
use axum::extract::ws::Message::Text;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use common::queue::{Entry, Queue};
use common::queue_tracker::QueueTracker;
use futures_util::SinkExt;
use futures_util::future::join_all;
use futures_util::stream::{SplitSink, SplitStream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tracing::debug;
use uuid::Uuid;

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
    match create_queue(registry.0, request.0).await {
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
struct QueueJoinRequest {
    players: Vec<Uuid>,
}

pub async fn queue_join(
    socket: WebSocket,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    queue_name: String,
) {
    let (sender, mut receiver): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) =
        socket.split();

    let sender_mutex: Arc<Mutex<SplitSink<WebSocket, Message>>> = Arc::new(Mutex::new(sender));

    let mut entry_ids: Vec<Uuid> = Vec::new();
    
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
        let id = on_join_request(
            &queue_name,
            join_request.unwrap(),
            queue_tracker.clone(),
            sender_mutex.clone(),
        )
        .await;
        
        if let Ok(id) = id {
            entry_ids.push(id);
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
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
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
        if queue_result.is_err() {
            println!(
                "Error waiting for queue result: {}",
                queue_result.unwrap_err()
            );
            return;
        }
        let message = serde_json::to_string(&queue_result.unwrap()).unwrap_or_default();

        sender
            .lock()
            .await
            .send(Text(message.into()))
            .await
            .unwrap();
    });

    Ok(entry_id)
    // queue tracker dropped
}

#[axum::debug_handler]
pub async fn get_queues_route(
    registry: State<Arc<Mutex<QueueTracker>>>,
) -> (StatusCode, Json<Vec<Value>>) {
    let tracker = registry.lock().await;

    let queues: &HashMap<String, Arc<Mutex<Queue>>> = tracker.get_queues();

    let futures = queues.iter().map(|(name, queue)| async move {
        let queue: MutexGuard<Queue> = queue.lock().await;
        let entries = queue.get_entries();

        json!({
            "name": name,
            "entries": entries,
        })
    });

    let queue_data: Vec<Value> = join_all(futures).await;

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
            let queue = q.lock().await;
            let entries = queue.get_entries();
            let matchmaker: Value = queue.matchmaker().serialize().unwrap_or_default();
            let response = json!({
                "name": name,
                "entries": entries,
                "matchmaker": matchmaker,
            });
            (StatusCode::OK, Json(response))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(Value::String(format!("Queue '{}' not found", name))),
        ),
    }
}
