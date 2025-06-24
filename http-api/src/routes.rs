use axum::Json;
use axum::extract::ws::Message::Text;
use axum::extract::ws::WebSocket;
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use common::queue::{Entry, Queue};
use common::queue_tracker::QueueTracker;
use futures_util::SinkExt;
use futures_util::future::join_all;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::error::Error;
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
    let (mut sender, mut receiver) = socket.split();

    if let Some(Ok(Text(text))) = receiver.next().await {
        match serde_json::from_str::<QueueJoinRequest>(&text) {
            Ok(request) => {
                let queue_tracker = queue_tracker.lock().await;
                if let Some(queue) = queue_tracker.get_queue(&queue_name) {
                    let join_fut = queue.lock().await.join_queue(Entry::new(request.players));
                    tokio::spawn(async move {
                        match join_fut.await {
                            Ok(Ok(response)) => {
                                if let Err(err) =
                                    sender.send(Text(response.to_string().into())).await
                                {
                                    eprintln!("Error sending join response: {}", err);
                                }
                            }
                            _ => {
                                if let Err(err) =
                                    sender.send(Text("Failed to join queue".into())).await
                                {
                                    eprintln!("Error sending join error: {}", err);
                                }
                            }
                        }
                    });
                }
            }
            Err(err) => {
                eprintln!("Error parsing join message: {}", err);
                let _ = sender.send(Text(err.to_string().into())).await;
            }
        }
    }
}

#[axum::debug_handler]
pub async fn get_queues_route(
    registry: State<Arc<Mutex<QueueTracker>>>,
) -> (StatusCode, Json<Vec<Value>>) {
    let tracker = registry.lock().await;

    let queues: &HashMap<String, Arc<Mutex<Queue>>> = tracker.get_queues();

    let futures = queues.iter().map(|(name, queue)| async move {
        let queue: MutexGuard<Queue> = queue.lock().await;
        let entries: Vec<&Entry> = queue.get_entries().values().collect();

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
            let entries: Vec<&Entry> = queue.get_entries().values().collect();
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
