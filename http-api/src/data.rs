use common::gamefinder::Game;
use common::queue::Entry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueData {
    name: String,
    entries: Vec<Entry>,
    matchmaker: Value,
}

impl QueueData {
    pub fn new(name: String, entries: Vec<Entry>, matchmaker: Value) -> Self {
        QueueData {
            name,
            entries,
            matchmaker,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct ErrorSocketResponse {
    pub(crate) context: Option<Uuid>,
    pub(crate) error: String,
}
#[derive(Serialize, Deserialize)]
pub struct SuccessSocketResponse {
    pub teams: Vec<Vec<Entry>>,
    pub game: Game,
}

#[derive(Serialize, Deserialize)]
pub enum SocketResponse {
    Success(SuccessSocketResponse),
    Error(ErrorSocketResponse),
}

#[derive(Serialize, Deserialize)]
pub struct QueueJoinRequest {
    pub id: Uuid,
    pub players: Vec<Uuid>,
    pub metadata: Map<String, Value>,
}
