mod data;
mod routes;

use axum::Router;
use axum::routing::{any, get, post};
use common::queue_tracker::QueueTracker;
use std::error::Error;
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    
    info!("Starting API server...");

    let queue_tracker = Arc::new(Mutex::new(QueueTracker::new("./queues")?));

    info!("Loaded all queues...");

    let app = Router::new()
        .route(
            "/api/v1/queue",
            post(routes::create_queue_route).get(routes::get_queues_route),
        )
        .route("/api/v1/queue/{name}", get(routes::get_queue))
        .route("/api/v1/queue/{name}/join", any(routes::ws_upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(queue_tracker.clone());

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;
    info!("Listening on: {}", listener.local_addr()?);

    let queue_tracker_clone = queue_tracker.clone();
    
    // Set up graceful shutdown on SIGTERM
    let shutdown_signal = async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        sigterm.recv().await;
        info!("SIGTERM received, waiting for all queues to become empty...");
        loop {
            let all_empty = {
                let tracker = queue_tracker_clone.lock().await;
                tracker.all_queues_empty().await
            };
            if all_empty {
                info!("All queues are empty. Proceeding with shutdown.");
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}
