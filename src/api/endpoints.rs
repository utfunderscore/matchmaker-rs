use crate::api::requests::requests::{CreateQueueRequest, QueueJoinRequest};
use crate::queues::queue_pool::QueuePool;
use actix_web::web::Json;
use actix_web::{get, put, web, HttpRequest, Responder};
use actix_ws::Message;
use serde_json::Value;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::sync::Mutex;

#[get("/queue/list")]
async fn get_queues(
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
) -> std::io::Result<impl Responder> {
    let pool = queue_pool.lock().await;

    let queues = pool
        .get_queues()
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(web::Json(queues))
}

#[put("/queue/{queue_id}/create")]
async fn create_queue(
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
    queue_id: web::Path<String>,
    queue_type: web::Query<CreateQueueRequest>,
    body: Json<Value>,
) -> std::io::Result<impl Responder> {
    let mut queue_pool = queue_pool.lock().await;

    let queue_id: String = queue_id.into_inner();

    let queue = queue_pool
        .create_queue(queue_id, queue_type.queue_type.clone(), body.0)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(Json(queue))
}

#[get("/queue/{queue_id}/join")]
async fn ws(
    req: HttpRequest,
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
    body: web::Payload,
) -> actix_web::Result<impl Responder> {
    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let session = Arc::new(Mutex::new(session));

    actix_web::rt::spawn({
        let thread_session = session.clone();

        async move {
            let mut session_lock = thread_session.lock().await;

            while let Some(Ok(msg)) = msg_stream.recv().await {
                match msg {
                    Message::Ping(bytes) => {
                        if session_lock.pong(&bytes).await.is_err() {
                            return;
                        }
                    }
                    Message::Text(text) => {
                        let join_request_result = serde_json::from_str::<QueueJoinRequest>(&text)
                            .map_err(|x| x.to_string());
                        if join_request_result.is_err() {
                            let _ = session_lock.text("Failed to parse join request.").await;
                            log::error!(
                                "Failed to parse join request: {}",
                                join_request_result.err().unwrap_or(String::from(""))
                            );
                        }
                    }
                    _ => break,
                }
            }
        }
    });

    Ok(response)
}
