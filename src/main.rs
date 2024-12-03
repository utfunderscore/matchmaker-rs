#[macro_use]
extern crate rocket;

use crate::api::endpoints;
use crate::game::game_provider::FakeGameProvider;
use crate::matchmaker::matchmaker::UnratedMatchmaker;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_pool::QueuePool;
use crate::queues::queue_ticker::QueueTicker;
use rocket::yansi::Paint;
use rocket::Config;
use uuid::Uuid;

mod api;
mod game;
mod matchmaker;
mod queues;

#[rocket::main]
async fn main() {
    let mut queue_pool = QueuePool::new();

    let entry = QueueEntry::from_vec(Vec::from([Uuid::new_v4()]));
    let mut queue1 = Queue::new(String::from("test"));

    queue1.add_team(entry).expect("test");

    queue_pool.add_ticker(QueueTicker::new(
        queue1,
        UnratedMatchmaker::new(1, 2),
        Box::new(FakeGameProvider {}),
    ));

    queue_pool.add_ticker(QueueTicker::new(
        Queue::new(String::from("test2")),
        UnratedMatchmaker::new(1, 2),
        Box::new(FakeGameProvider {}),
    ));

    let config = Config {
        port: 8001,
        address: std::net::Ipv4Addr::new(127, 0, 0, 1).into(),
        ..Config::debug_default()
    };

    rocket::custom(&config)
        .manage(queue_pool)
        .mount("/", routes![endpoints::get_queues, endpoints::join])
        .launch()
        .await
        .expect("TODO: panic message");
}
