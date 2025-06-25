mod data;
mod routes;

use axum::Router;
use axum::routing::{any, get, post};
use common::queue_tracker::QueueTracker;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let queue_tracker = Arc::new(Mutex::new(QueueTracker::new("./queues")?));

    let app = Router::new()
        .route(
            "/api/v1/queue",
            post(routes::create_queue_route).get(routes::get_queues_route),
        )
        .route("/api/v1/queue/{name}", get(routes::get_queue))
        .route("/api/v1/queue/{name}/join", any(routes::ws_upgrade))
        .with_state(queue_tracker);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
