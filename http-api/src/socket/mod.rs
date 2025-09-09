use axum::extract::ws::Message::Text;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use common::queue::{Entry, QueueResult};
use common::queue_tracker::QueueTracker;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::log::error;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct QueueJoinRequest {
    id: Uuid,
    players: Vec<Uuid>,
    pub metadata: Map<String, Value>,
}

#[axum::debug_handler]
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    Path(queue): Path<String>,
) -> Response {
    ws.on_upgrade(|x| queue_join(x, queue_tracker, queue))
}

pub async fn queue_join(
    socket: WebSocket,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    queue_name: String,
) {
    let (mut sender, mut receiver): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) =
        socket.split();

    let mut entry_ids: Vec<Uuid> = Vec::new();

    // Check if the queue exists before
    if queue_tracker
        .0
        .lock()
        .await
        .get_queue(&queue_name)
        .is_none()
    {
        let _ = send_err(
            &mut sender,
            None,
            &format!("Queue '{queue_name}' does not exist"),
        )
        .await;
        return;
    }

    let sender_mutex: Arc<Mutex<SplitSink<WebSocket, Message>>> = Arc::new(Mutex::new(sender));

    while let Some(Ok(Text(text))) = receiver.next().await {
        debug!("Received join request: {}", text);

        let join_request: Result<QueueJoinRequest, _> = serde_json::from_str(&text);

        let Ok(join_request) = join_request else {
            let mut sender = sender_mutex.lock().await;

            let _ = send_err(
                &mut sender,
                None,
                "Failed to read join request: Invalid Json",
            );
            continue;
        };

        let entry_id = join_request.id;

        let join_result: Result<Uuid, String> = on_join_request(
            &queue_name,
            join_request,
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
                send_err(&mut sender, Some(&entry_id), &format!("Failed to join queue: {err}")).await;
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
    let entry = Entry::new(join_request.id, join_request.players, join_request.metadata);
    let entry_id = entry.id();

    let receiver = queue.join_queue(entry)?;

    if queue.get_entry(&entry_id).is_some() {
        return Err(String::from("Entry already exists with that id").into());
    }

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

async fn send_err(sender: &mut SplitSink<WebSocket, Message>, entry_id: Option<&Uuid>, err: &str) {
    match entry_id {
        None => {
            send_json(sender, &json!({"error": err})).await;
        }
        Some(uuid) => {
            send_json(sender, &json!({"error": err, "context": uuid.to_string()})).await;
        }
    }
}

async fn send_json<T>(sender: &mut SplitSink<WebSocket, Message>, json: &T)
where
    T: ?Sized + Serialize,
{
    match serde_json::to_string(&json) {
        Ok(str) => {
            let _ = sender.send(Text(str.into())).await;
        }
        Err(_) => {
            error!("Failed to send json")
        }
    }
}
