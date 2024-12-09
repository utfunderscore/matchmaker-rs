use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct QueueJoinRequest {
    pub players: Vec<Uuid>,
    pub attributes: Value,
}
