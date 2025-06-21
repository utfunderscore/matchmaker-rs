mod routes;

use axum::routing::post;
use axum::Router;
use common::algo::flexible;
use common::registry::Registry;
use std::error::Error;
use std::sync::{Arc};
use tokio::sync::Mutex;
use common::codec::Codec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let registry = Registry::new();
    let mut codec = Codec::new();

    codec.register_deserializer("flexible", flexible::DESERIALIZER);
    
    let app_data = AppData {
        registry,
        codec,
    };
    
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
