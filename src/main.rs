use crate::api::queue_endpoints;
use crate::api::queue_sockets;
use crate::game::providers::FakeGame;
use crate::matchmaker::implementations::Unrated;
use crate::matchmaker::serializer::Registry;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_store::{FlatFile, QueueStore};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Local;
use log::{error, info, warn, LevelFilter};
use serde_json::Value;
use std::io::Error;
use std::io::ErrorKind::Other;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use web::Data;

mod api;
mod game;
mod matchmaker;
mod queues;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matchmaker_registry = Registry::new();
    let queue_store = FlatFile::new(String::from("queues.json"), matchmaker_registry);
    let mut queue_pool = queue_store.load().map_err(|x| Error::new(Other, x));
    if let Err(err) = queue_pool {
        error!("Failed to load queue: {}", err);
        return Err(err);
    }
    let mut queue_pool = queue_pool?;


    queue_pool.add_creator(
        String::from("unrated"),
        Box::new(Unrated::create_unrated_queue),
    );

    let test = QueueEntry {
        id: Uuid::new_v4(),
        players: vec![Uuid::new_v4()],
        attributes: Value::default(),
    };

    println!(
        "Queue join request: {:?}",
        serde_json::to_string(&test).unwrap()
    );

    let queue_data = Arc::new(Mutex::new(queue_pool));

    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    HttpServer::new({
        let inner_queue_data = queue_data.clone();
        move || {
            let data = Data::new(inner_queue_data.clone());

            App::new()
                .wrap(Logger::default())
                .app_data(data)
                .service(queue_endpoints::get_queues)
                .service(queue_endpoints::create_queue)
                .service(queue_sockets::join_queue_socket)
        }
    })
    .workers(4)
    .bind(("localhost", 8383))?
    .run()
    .await?;

    info!("Saving queues...");

    queue_store
        .save(queue_data.clone())
        .await
        .map_err(|x| Error::new(Other, x))?;

    Ok(())
}
