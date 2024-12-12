use crate::api::endpoints;
use crate::game::game_provider::FakeGameProvider;
use crate::matchmaker::matchmaker::UnratedMatchmaker;
use crate::matchmaker::serializer::SerializerRegistry;
use crate::queues::queue_store::{FlatFileQueueStore, QueueStore};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::io::Error;
use std::io::ErrorKind::Other;
use std::sync::Arc;
use tokio::sync::Mutex;
use web::Data;

mod api;
mod game;
mod matchmaker;
mod queues;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let queue_store = FlatFileQueueStore::new(String::from("test.json"), SerializerRegistry::new());
    let mut queue_pool = queue_store.load().map_err(|x| Error::new(Other, x))?;

    queue_pool.add_creator(
        String::from("unrated"),
        Box::new(UnratedMatchmaker::create_unrated_queue),
    );

    let queue_data = Arc::new(Mutex::new(queue_pool));

    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");

    env_logger::init();

    HttpServer::new({
        let inner_queue_data = queue_data.clone();
        move || {
            let data = Data::new(inner_queue_data.clone());

            App::new()
                .wrap(Logger::default())
                .app_data(data)
                .service(endpoints::get_queues)
                .service(endpoints::create_queue)
                .service(endpoints::ws)
        }
    })
    .workers(4)
    .bind(("localhost", 8383))?
    .run()
    .await?;

    queue_store
        .save(queue_data.clone())
        .await
        .map_err(|x| Error::new(Other, x))?;

    Ok(())
}
