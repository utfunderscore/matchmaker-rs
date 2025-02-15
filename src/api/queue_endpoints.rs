use crate::queues::queue_pool::QueuePool;
use actix_web::web::Json;
use actix_web::{get, put, web, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::io::ErrorKind::Other;
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
        .map_err(|e| Error::new(Other, e.to_string()))?;

    Ok(web::Json(queues))
}

#[derive(Debug, Deserialize)]
struct QueueType {
    queue_type: String,
}

#[put("/queue/{queue_id}/create")]
async fn create_queue(
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
    queue_id: web::Path<String>,
    queue_type: web::Query<QueueType>,
    body: Json<Value>,
) -> std::io::Result<impl Responder> {
    let mut queue_pool = queue_pool.lock().await;

    let queue_id: String = queue_id.into_inner();

    let queue_type = queue_type.into_inner().queue_type;


    if queue_pool.queue_exists(&queue_id) {
        return Err(Error::new(Other, "Queue already exists with that name"));
    }

    let queue = queue_pool
        .create_queue(queue_id, &queue_type, &body.into_inner())
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(Json(queue))
}
