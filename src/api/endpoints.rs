use crate::queues::queue_pool::QueuePool;
use actix_web::web::Json;
use actix_web::{get, put, web, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

#[get("/queue/list")]
async fn get_queues(
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
) -> std::io::Result<impl Responder> {
    let pool = queue_pool
        .lock()
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let queues = pool
        .get_queues()
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(web::Json(queues))
}

#[derive(Deserialize)]
struct CreateQueueQuery {
    queue_type: String,
}

#[put("/queue/{queue_id}/create")]
async fn create_queue(
    queue_pool: web::Data<Arc<Mutex<QueuePool>>>,
    queue_id: web::Path<String>,
    queue_type: web::Query<CreateQueueQuery>,
    body: web::Json<Value>,
) -> std::io::Result<impl Responder> {
    let mut queue_pool = queue_pool
        .lock()
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let queue_id: String = queue_id.into_inner();

    let queue = queue_pool
        .create_queue(queue_id, queue_type.queue_type.clone(), body.0)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(Json(queue))
}
