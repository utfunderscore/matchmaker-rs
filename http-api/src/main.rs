mod routes;

use axum::Router;
use axum::routing::{get, post};
use common::algo::flexible;
use common::codec::Codec;
use common::registry::Registry;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let mut codec = Codec::new();

    codec.register_deserializer("flexible", flexible::DESERIALIZER);

    let registry = Registry::new("./data", codec);

    let app_data = Arc::new(Mutex::new(registry));

    let app = Router::new()
        .route("/api/v1/queue", post(routes::create_queue_route).get(routes::get_queues_route))
        .route("/api/v1/queue/{name}", get(routes::get_queue))
        .with_state(app_data);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
