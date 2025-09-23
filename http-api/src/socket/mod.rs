use crate::data::{QueueError, QueueJoinRequest};
use axum::extract::ws::Message::Text;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use common::entry::{Entry, EntryId};
use common::queue::QueueResult;
use common::queue_tracker::QueueTracker;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use crate::state::AppState;

#[axum::debug_handler]
pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    app_state: State<AppState>,
    Path(queue): Path<String>,
) -> Response {
    let queue_tracker = app_state.0.queue_tracker;
    ws.on_upgrade(|x| handle_socket(x, queue_tracker, queue))
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
            // send_socket(sender, Err("Failed to receive initial message".to_string())).await;
            info!("Socket error occurred");
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

        let queue_name = &queue_name;
        let id = queue_join_request.id.clone();

        debug!("Parsed join request: {:?}", queue_join_request);
        let result = tokio::select! {
            res = join_queue(queue_name, queue_join_request, queue_tracker.clone()) => Some(res),
            msg = async {
                loop {
                    match receiver.next().await {
                        None => break None,
                        Some(Err(_)) => break None,
                        Some(Ok(_)) => continue,
                    }
                }
            } => msg,
        };
        if let Some(result) = result {
            println!("Sending: {:?}", result);
            send_socket(sender, result).await;
        } else {
            let mut queue_tracker = queue_tracker.lock().await;
            queue_tracker.leave(queue_name, EntryId(id)).await
        }
    });
}

pub async fn join_queue(
    queue_name: &str,
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
        .join(queue_name, entry)
        .await
        .map_err(|x| x.to_string())?;

    drop(tracker_guard);

    QueueTracker::tick_task(queue_tracker, queue_name).await;

    debug!("Joined queue, waiting for queue result...");

    let result = receiver.await.map_err(|x| x.to_string())??;
    Ok(result)
}

async fn send_socket(
    mut sender: SplitSink<WebSocket, Message>,
    socket_response: Result<QueueResult, String>,
) {
    let socket_response = socket_response.map_err(|x| QueueError::new(x));

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
