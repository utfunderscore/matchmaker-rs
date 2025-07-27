use std::sync::Arc;
use tokio::sync::Mutex;
use common::gamefinder::GameFinder;
use common::queue_tracker::QueueTracker;

pub struct AppState {
    pub queue_tracker: Arc<Mutex<QueueTracker>>,
    pub game_finder: Arc<Mutex<GameFinder>>,
}