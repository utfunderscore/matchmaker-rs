mod routes;

use axum::Router;
use axum::routing::post;
use common::algo::flexible;
use common::codec::Codec;
use common::registry::Registry;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let registry = Registry::new();
    let mut codec = Codec::new();

    codec.register_deserializer("flexible", flexible::DESERIALIZER);

    let app_data = AppData { registry, codec };

    let app_data = Arc::new(Mutex::new(app_data));

    let app = Router::new()
        .route("/api/v1/queue", post(routes::create_queue_route))
        .with_state(app_data);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

struct AppData {
    registry: Registry,
    codec: Codec,
}
