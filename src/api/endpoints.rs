use crate::game::game::Game;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_pool::QueuePool;
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::sync::oneshot::Receiver;
use rocket::State;
use serde::Deserialize;
use serde_json::{Error, Value};
use std::fmt::Write;
use uuid::Uuid;
use ws::{Message, WebSocket};

#[get("/queue/list")]
pub fn get_queues(queue_pool_state: &State<QueuePool>) -> (Status, Json<Vec<Queue>>) {
    (Status::Ok, Json(queue_pool_state.get_queues()))
}

#[derive(Deserialize)]
struct Players {
    players: Vec<Uuid>,
}

#[get("/queue/<name>/join")]
pub fn join<'b>(name: String, queue_pool: &'b State<QueuePool>, ws: WebSocket) -> ws::Channel<'b> {
    ws.channel(move |mut stream| {
        Box::pin(async move {
            if !queue_pool.queue_exists(&name) {
                let _ = stream
                    .send(Message::Text(String::from("Invalid queue")))
                    .await;
                return Ok(());
            }

            let first_message = stream.next().await;
            if first_message.is_none() {
                return Ok(());
            }
            let first_message = first_message.unwrap().unwrap();
            if !first_message.is_text() {
                return Ok(());
            }

            let json_string = first_message.into_text()?;

            let players: Result<Vec<Uuid>, Error> = serde_json::from_str::<Vec<Uuid>>(&json_string);
            if let Err(x) = players {
                let _ = stream.send(Message::Text(x.to_string())).await;
                return Ok(());
            }
            let entry = QueueEntry::from_vec(players.unwrap());

            let join_result: Result<Receiver<Result<Game, String>>, String> =
                queue_pool.join_queue(name, entry);

            if join_result.is_err() {
                let _ = stream.send(Message::Text(join_result.unwrap_err())).await;
                return Ok(());
            }

            let join_result_receiver = join_result.unwrap();

            let join_result = join_result_receiver.await.unwrap();

            match join_result {
                Ok(game) => {
                    let game_json = serde_json::to_string(&game).unwrap();

                    let _ = stream.send(Message::Text(game_json)).await;
                }
                Err(err) => {}
            }

            Ok(())
        })
    })
}

// #[get("/queue/<name>/join", data = "<input>")]
// pub fn join_queue(
//     queue_pool: &State<QueuePool>,
//     name: String,
//     input: Json<Vec<Uuid>>,
// ) -> Result<Json<Queue>, Custom<String>> {
//     let queue_entry = QueueEntry::from_vec(input.0);
//
//     let join_result: Result<bool, String> = queue_pool.join_queue(name.clone(), queue_entry);
//
//     if let Err(error) = join_result {
//         println!("{}", error);
//         return Err(Custom(Status::InternalServerError, error));
//     }
//
//     Ok(Json(queue_pool.get_queue_copy(name.clone()).unwrap()))
// }
