mod data;
mod game_routes;
mod queue_routes;
mod socket;
mod state;

use crate::state::AppState;
use axum::Router;
use axum::routing::{any, get, post, put};
use common::gamefinder::{GameFinder, GameFinderSettings};
use common::queue_tracker::QueueTracker;
use std::error::Error;
use std::sync::{Arc};
use tokio::net::TcpListener;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Print build info at startup

    println!(
        "Build Information:\n  Build time: {}\n  Git commit hash: {}",
        env!("BUILD_TIME"),
        env!("GIT_HASH")
    );

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            println!("No environment variable for 'RUST_LOG' found, defaulting to INFO");
            EnvFilter::new("info")
        }))
        .init();

    info!("Loading config...");

    let finder_settings = Arc::new(RwLock::new(
        GameFinderSettings::load_or_create_config("config.json").await?,
    ));
    let game_finder = GameFinder::new(finder_settings.clone());

    info!("Initializing queue tracker...");
    let queue_tracker = QueueTracker::from_file(game_finder).await;
    let queue_tracker_clone = queue_tracker.clone();

    let state = AppState {
        finder_settings,
        queue_tracker,
    };

    info!("Loaded all queues...");

    let app = Router::new()
        .route(
            "/api/v1/queue",
            post(queue_routes::create_queue_route).get(queue_routes::get_queues_route),
        )
        .route("/api/v1/queue/{name}", get(queue_routes::get_queue))
        .route("/api/v1/queue/{name}/join", any(socket::ws_upgrade))
        .route("/api/v1/finder", put(game_routes::update_settings))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = TcpListener::bind("[::]:8080").await?;
    info!("Listening on: {}", listener.local_addr()?);

    // Set up graceful shutdown on SIGTERM and SIGINT
    let shutdown_signal = async move {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
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
            let all_empty = { tracker.all_queues_empty().await };
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
        .await
        .map_err(|x| format!("Failed to service http api: {}", x.to_string()))?;

    Ok(())
}
