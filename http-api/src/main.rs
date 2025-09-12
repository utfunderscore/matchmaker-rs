mod data;
mod queue_routes;
mod socket;

use axum::Router;
use axum::routing::{any, get, post};
use common::queue_tracker::QueueTracker;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing::{info};
use tracing_subscriber::EnvFilter;
use common::gamefinder::{GameFinder, GameFinderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Loading config...");


    let game_finder_config = GameFinderConfig::load_or_create_config("config.toml").await?;

    let game_finder = Arc::new(Mutex::new(GameFinder::new(game_finder_config)));

    info!("Initializing queue tracker...");
    let queue_tracker = QueueTracker::from_file().await;
    let queue_tracker_clone = queue_tracker.clone();

    info!("Loaded all queues...");

    let app = Router::new()
        .route(
            "/api/v1/queue",
            post(queue_routes::create_queue_route).get(queue_routes::get_queues_route),
        )
        .route("/api/v1/queue/{name}", get(queue_routes::get_queue))
        .route("/api/v1/queue/{name}/join", any(socket::ws_upgrade))
        .layer(TraceLayer::new_for_http())
        .with_state(queue_tracker);

    let listener = TcpListener::bind("[::]:8080").await?;
    info!("Listening on: {}", listener.local_addr()?);


   // Set up graceful shutdown on SIGTERM and SIGINT
    let shutdown_signal = async move {
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("SIGTERM received, waiting for all queues to become empty...");
            }
            _ = sigint.recv() => {
                info!("SIGINT received, waiting for all queues to become empty...");
            }
        }

        loop {
            let tracker = queue_tracker_clone.lock().await;
            let all_empty = {
                tracker.all_queues_empty().await
            };
            if all_empty {
                info!("All queues are empty. Proceeding with shutdown.");
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            tracker.save_to_file().await;
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}


