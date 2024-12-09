use crate::game::game::Game;
use crate::game::game_provider::GameProvider;
use crate::matchmaker::matchmaker::Matchmaker;
use crate::queues::queue::Queue;
use crate::queues::queue_entry::QueueEntry;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use uuid::Uuid;

pub struct QueueTicker {
    queue: Queue,
    matchmaker: Box<dyn Matchmaker + Send + Sync>,
    game_producer: Box<dyn GameProvider + Send + Sync>,
    entry_channels: HashMap<Uuid, Sender<Result<Game, String>>>,
}
impl QueueTicker {
    pub fn new(
        queue: Queue,
        matchmaker: Box<dyn Matchmaker + Send + Sync>,
        game_producer: Box<dyn GameProvider + Send + Sync>,
    ) -> Arc<Mutex<Self>> {
        let ticker = Self {
            queue,
            matchmaker,
            game_producer,
            entry_channels: HashMap::new(),
        };

        let ticker_arc = Arc::new(Mutex::new(ticker));
        let ticker_ref = Arc::downgrade(&ticker_arc);

        tokio::spawn(async move {
            loop {
                let ticker = ticker_ref.upgrade();

                match ticker {
                    None => break,
                    Some(ticker) => ticker.lock().unwrap().tick(),
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        ticker_arc
    }

    fn tick(&mut self) {
        let matchmaker_result = self.matchmaker.matchmake(&self.queue.in_queue);
        if matchmaker_result.is_err() {
            return;
        }
        let teams = matchmaker_result.unwrap();

        println!("teams {:?}", teams);

        let game_result = self.game_producer.get_game_server();
        if game_result.is_err() {
            return;
        }
        let game = game_result.unwrap();

        for team in teams {
            for entry_id in team {
                if let Err(e) = self.queue.remove_team(&entry_id) {
                    eprintln!("Failed to remove team from the queue: {}", e);
                } else {
                    self.try_notify_socket(&entry_id, game.clone());
                }
            }
        }
    }

    pub fn store() {}

    fn try_notify_socket(&mut self, entry_id: &Uuid, game: Game) {
        let sender = self.entry_channels.get(entry_id);
        if sender.is_none() {
            return;
        }

        let removed = self.entry_channels.remove(entry_id).unwrap();
        removed.send(Ok(game)).unwrap()
    }

    pub fn add_team(
        &mut self,
        queue_entry: QueueEntry,
    ) -> Result<Receiver<Result<Game, String>>, String> {
        self.matchmaker.validate_entry(&queue_entry)?;

        let (sender, receiver) = channel::<Result<Game, String>>();

        self.add_channel(queue_entry.id, sender);

        self.get_queue_mut().add_team(queue_entry)?;

        Ok(receiver)
    }

    pub fn add_channel(&mut self, entry_id: Uuid, sender: Sender<Result<Game, String>>) {
        self.entry_channels.insert(entry_id, sender);
    }

    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }

    pub fn get_queue_mut(&mut self) -> &mut Queue {
        &mut self.queue
    }
}
