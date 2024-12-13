use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_pool::QueuePool;
use actix_web::web::Data;
use actix_web::{get, web, HttpRequest, Responder};
use actix_ws::{Message, Session};
use log::info;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use uuid::Uuid;

#[get("/queue/{queue_id}/join")]
pub async fn join_queue_socket(
    req: HttpRequest,
    queue_id: web::Path<String>,
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let session = Arc::new(Mutex::new(session));

    actix_web::rt::spawn({
        let thread_session = session.clone();

        async move {
            while let Some(Ok(msg)) = msg_stream.recv().await {
                match msg {
                    Message::Ping(bytes) => {
                        let mut session_lock = thread_session.lock().await;
                        if session_lock.pong(&bytes).await.is_err() {
                            return;
                        }
                    }
                    Message::Text(text) => {
                        info!("Received websocket message {}", text);

                        let join_request_result =
                            serde_json::from_str::<QueueEntry>(&text).map_err(|x| x.to_string());
                        if join_request_result.is_err() {
                            let mut session_lock = thread_session.lock().await;
                            let _ = session_lock.text("Failed to parse join request.").await;
                            log::error!(
                                "Failed to parse join request: {}",
                                join_request_result.err().unwrap_or_default()
                            );
                            return;
                        }

                        queue_join_request(
                            queue_id.clone(),
                            join_request_result.unwrap(),
                            queue_pool.clone(),
                            thread_session.clone(),
                        );
                    }
                    _ => break,
                }
            }
        }
    });

    Ok(response)
}

fn queue_join_request(
    queue_name: String,
    queue_entry: QueueEntry,
    queue_pool_data: Data<Arc<Mutex<QueuePool>>>,
    channel: Arc<Mutex<Session>>,
) {
    actix_web::rt::spawn(async move {
        info!(
            "{} players have joined the queue {}",
            queue_entry.players.len(),
            queue_name
        );

        let queue_entry_id = &queue_entry.id.clone();

        let queue_pool_data = queue_pool_data.clone();

        let queue_pool = queue_pool_data.lock().await;

        let receiver = queue_pool.join_queue(&queue_name, queue_entry);
        if receiver.is_err() {
            let mut channel = channel.lock().await;
            let receiver_err = receiver.err().unwrap_or(String::new());
            info!("Failed to join queue: {}", receiver_err);
            let _ = channel
                .text(format!("Failed to join queue: {receiver_err}"))
                .await;
            return;
        }

        drop(queue_pool);

        let receiver = receiver.unwrap();

        let queue_join_result = receiver.await;

        if queue_join_result.is_err() {
            let queue_pool = queue_pool_data.lock().await;

            handle_queue_error(
                queue_name,
                queue_entry_id,
                &mut channel.lock().await,
                &queue_pool,
            )
            .await;
            return;
        }
        let queue_join_result = queue_join_result.unwrap();
        if queue_join_result.is_err() {
            let queue_pool = queue_pool_data.lock().await;
            handle_queue_error(
                queue_name,
                queue_entry_id,
                &mut channel.lock().await,
                &queue_pool,
            )
            .await;
            return;
        }
        let queue_join_result = queue_join_result.unwrap();

        let mut channel = channel.lock().await;

        let result_json = serde_json::to_string(&queue_join_result);
        if result_json.is_err() {
            let queue_pool = queue_pool_data.lock().await;
            handle_queue_error(queue_name, queue_entry_id, &mut channel, &queue_pool).await;
            return;
        }

        let _ = channel.text(result_json.unwrap()).await;
    });
}

async fn handle_queue_error(
    queue_name: String,
    queue_entry_id: &Uuid,
    channel: &mut MutexGuard<'_, Session>,
    queue_pool: &MutexGuard<'_, QueuePool>,
) {
    let _ = channel.text("Failed whilst finding a game").await;
    println!("sent");
    let _ = queue_pool.leave_queue(&queue_name, queue_entry_id);
}
