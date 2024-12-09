use crate::api::endpoints;
use crate::game::game_provider::FakeGameProvider;
use crate::matchmaker::matchmaker::UnratedMatchmaker;
use crate::queues::queue::Queue;
use crate::queues::queue_pool::QueuePool;
use crate::queues::queue_ticker::QueueTicker;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
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

    let mut queue_pool = QueuePool::new();

    queue_pool.add_creator(
        String::from("unrated"),
        Box::new(|name, body| {
            let team_size = body
                .get("team_size")
                .ok_or("No queue size provided")?
                .as_u64()
                .ok_or("Invalid team size, type must be integer")?;

            let number_of_teams = body
                .get("number_of_teams")
                .ok_or("No queue size provided")?
                .as_u64()
                .ok_or("Invalid number of teams, type must be integer")?;

            Ok(QueueTicker::new(
                Queue::new(name),
                UnratedMatchmaker::new(team_size, number_of_teams),
                Box::new(FakeGameProvider {}),
            ))
        }),
    );

    let queue_pool_mutex = Arc::new(Mutex::new(queue_pool));

    let weak_ref = Arc::downgrade(&queue_pool_mutex);

    let queue_data = web::Data::new(queue_pool_mutex);

    std::env::set_var("RUST_LOG", "actix_web=info,actix_server=info");

    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(queue_data.clone())
            .service(endpoints::get_queues)
            .service(endpoints::create_queue)
    })
    .bind(("localhost", 8383))?
    .run()
    .await
}
