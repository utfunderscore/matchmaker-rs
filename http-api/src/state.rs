use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use common::gamefinder::GameFinderSettings;
use common::queue_tracker::QueueTracker;

#[derive(Clone)]
pub struct AppState {
    pub finder_settings: Arc<RwLock<GameFinderSettings>>,
    pub queue_tracker: Arc<Mutex<QueueTracker>>
}