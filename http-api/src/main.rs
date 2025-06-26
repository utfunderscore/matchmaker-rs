mod data;
mod routes;

use axum::Router;
use axum::routing::{any, get, post};
use common::queue_tracker::QueueTracker;
use tower_http::trace::TraceLayer;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    
    info!("Starting API server...");

    let queue_tracker = Arc::new(Mutex::new(QueueTracker::new("./queues")?));

    let app = Router::new()
        .route(
            "/api/v1/queue",
            post(routes::create_queue_route).get(routes::get_queues_route),
        )
        .route("/api/v1/queue/{name}", get(routes::get_queue))
        .route("/api/v1/queue/{name}/join", any(routes::ws_upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(queue_tracker);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
