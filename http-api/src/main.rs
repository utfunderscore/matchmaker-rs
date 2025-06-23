mod routes;
mod data;

use axum::Router;
use axum::routing::{get, post};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use common::queue_tracker::QueueTracker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();


    let queue_tracker = Arc::new(Mutex::new(QueueTracker::new("./queues")));

    let app = Router::new()
        .route("/api/v1/queue", post(routes::create_queue_route).get(routes::get_queues_route))
        .route("/api/v1/queue/{name}", get(routes::get_queue))
        .with_state(queue_tracker);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
