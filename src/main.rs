use crate::api::endpoints;
use crate::game::game_provider::FakeGameProvider;
use crate::matchmaker::matchmaker::UnratedMatchmaker;
use crate::matchmaker::serializer::SerializerRegistry;
use crate::queues::queue::Queue;
use crate::queues::queue_store::{FlatFileQueueStore, QueueStore};
use crate::queues::queue_ticker::QueueTicker;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind::Other;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod api;
mod game;
mod matchmaker;
mod queues;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let test = api::requests::requests::QueueJoinRequest {
        players: vec![Uuid::new_v4()],
        attributes: Default::default(),
    };

    println!("{:?}", serde_json::to_string(&test).unwrap());

    let file = File::open(String::from("test.json"))
        .or_else(|_| File::create(String::from("./test.json")))?;

    let queue_store = FlatFileQueueStore::new(file, SerializerRegistry::new());
    let mut queue_pool = queue_store.load().map_err(|x| Error::new(Other, x))?;

    let _ = queue_pool.add_ticker(QueueTicker::new(
        Queue::new(String::from("test")),
        UnratedMatchmaker::new(1, 2),
        Box::new(FakeGameProvider {}),
    ));

    queue_pool.add_creator(
        String::from("unrated"),
        Box::new(UnratedMatchmaker::create_unrated_queue),
    );

    let queue_pool_mutex = Arc::new(Mutex::new(queue_pool));
    let weak_queue_pool = Arc::downgrade(&queue_pool_mutex);
    let queue_data = web::Data::new(queue_pool_mutex);

    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");

    env_logger::init();

    HttpServer::new({
        let queue_data = queue_data.clone();

        move || {
            App::new()
                .wrap(Logger::default())
                .app_data(queue_data.clone())
                .service(endpoints::get_queues)
                .service(endpoints::create_queue)
        }
    })
    .bind(("localhost", 8383))?
    .run()
    .await?;

    queue_store
        .save(queue_data)
        .map_err(|x| Error::new(Other, x))?;

    Ok(())
}
