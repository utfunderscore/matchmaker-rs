use crate::data::QueueJoinRequest;
use axum::extract::ws::Message::Text;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use common::entry::Entry;
use common::queue::QueueResult;
use common::queue_tracker::QueueTracker;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

#[axum::debug_handler]
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    queue_tracker: State<Arc<Mutex<QueueTracker>>>,
    Path(queue): Path<String>,
) -> Response {
    ws.on_upgrade(|x| handle_socket(x, queue_tracker.0, queue))
}

pub async fn handle_socket(
    socket: WebSocket,
    queue_tracker: Arc<Mutex<QueueTracker>>,
    queue_name: String,
) {
    info!("Handling socket for queue: {}", queue_name);

    let (sender, mut receiver): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) =
        socket.split();

    tokio::spawn(async move {
        debug!("Waiting for initial message from client...");
        let Some(Ok(Text(text))) = receiver.next().await else {
            send_socket(sender, Err("Failed to receive initial message".to_string())).await;
            return;
        };

        debug!("Received initial message: {}", text);
        let queue_join_request: QueueJoinRequest = match serde_json::from_str(&text) {
            Ok(request) => request,
            Err(err) => {
                send_socket(
                    sender,
                    Err(format!("Failed to parse join request: {}", err)),
                )
                .await;
                return;
            }
        };

        debug!("Parsed join request: {:?}", queue_join_request);
        let result = join_queue(queue_name, queue_join_request, queue_tracker).await;
        send_socket(sender, result).await;
    });
}

pub async fn join_queue(
    queue_name: String,
    queue_join_request: QueueJoinRequest,
    queue_tracker: Arc<Mutex<QueueTracker>>,
) -> Result<QueueResult, String> {
    debug!("Waiting for queue tracker lock...");
    let mut tracker_guard = queue_tracker.lock().await;

    let entry = Entry::new(
        queue_join_request.id,
        queue_join_request.players,
        queue_join_request.metadata,
    );

    let receiver = tracker_guard
        .join(&queue_name, entry)
        .await
        .map_err(|x| x.to_string())?;
    drop(tracker_guard);
    debug!("Joined queue, waiting for queue result...");

    let result = receiver.await.map_err(|x| x.to_string())??;
    Ok(result)
}

async fn send_socket(
    mut sender: SplitSink<WebSocket, Message>,
    socket_response: Result<QueueResult, String>,
) {
    match serde_json::to_string(&socket_response) {
        Ok(json) => {
            match sender.send(Text(json.into())).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to send socket response: {}", err);
                    return;
                }
            };
        }
        Err(err) => {
            error!("Failed to serialize socket response: {}", err);
            return;
        }
    };
}
