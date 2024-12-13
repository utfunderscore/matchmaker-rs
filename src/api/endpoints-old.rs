use crate::api::requests::requests::QueueJoinRequest;
use crate::game::game::Game;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use crate::queues::queue_pool::QueuePool;
use rocket::futures::{FutureExt, SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::select;
use rocket::tokio::sync::oneshot::Receiver;
use rocket::{tokio, State};
use serde::Deserialize;
use serde_json::{Error, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;
use ws::{Message, WebSocket};

#[get("/queue/list")]
pub fn get_queues(
    queue_pool_state: &State<QueuePool>,
) -> (Status, Json<Result<Vec<Queue>, String>>) {
    match queue_pool_state.get_queues() {
        Ok(queues) => (Status::Ok, Json(Ok(queues))),
        Err(e) => (Status::InternalServerError, Json(Err(e))),
    }
}

#[put("/queue/<id>/create?<creator>", data = "<queue_data>")]
pub fn create_queue(
    queue_pool_mutex: &State<Arc<Mutex<QueuePool>>>,
    creator: &str,
    id: &str,
    queue_data: Json<Value>,
) -> Json<Result<Queue, String>> {
    let queue_pool_lock = queue_pool_mutex.lock();

    if let Err(e) = queue_pool_lock {
        return Json(Err(e.to_string()));
    }
    let mut queue_pool: QueuePool = queue_pool_lock.unwrap();

    Json(queue_pool.create_queue(String::from(id), String::from(creator), queue_data.0))
}

#[derive(Deserialize)]
struct Players {
    players: Vec<Uuid>,
}
async fn pause() {
    tokio::time::sleep(Duration::from_secs(5)).await;
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
            // Await and r
            // Await and read the first message from the websocket

            let first_message = stream.next().await;
            if first_message.is_none() {
                return Ok(());
            }
            let first_message = first_message.unwrap()?;

            // First message should be a json string of a queue join request
            let json_string = first_message.into_text()?;
            let join_request: Result<QueueJoinRequest, Error> =
                serde_json::from_str::<QueueJoinRequest>(&json_string);
            if let Err(x) = join_request {
                let _ = stream.send(Message::Text(x.to_string())).await;
                return Ok(());
            }
            let join_request: QueueJoinRequest = join_request.unwrap();

            // Generate queue entry based on provided data
            let entry = QueueEntry::new(join_request.players, join_request.attributes);

            let entry_id = entry.id;

            let join_result: Result<Receiver<Result<Game, String>>, String> =
                queue_pool.join_queue(&name, entry);

            if join_result.is_err() {
                let _ = stream.send(Message::Text(join_result.unwrap_err())).await;
                return Ok(());
            }

            let join_result_receiver = join_result.unwrap();

            let next_message = stream.next().fuse();

            select! {
                result = join_result_receiver => {

                }
                _ = next_message => {

                    if let Err(err) = queue_pool.leave_queue(&name, &entry_id) {
                        eprintln!("Could not leave queue on websocket close {err}")
                    }

                    let _ = stream.close(None).await;
                },
            }

            Ok(())
        })
    })
}
