use common::queue_tracker::QueueTracker;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub queue_tracker: Arc<Mutex<QueueTracker>>
}