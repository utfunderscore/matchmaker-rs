use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct EntryId(pub Uuid);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: EntryId,
    pub players: Vec<Uuid>,
    #[serde(skip)]
    pub time_queued: DateTime<Utc>,
    pub metadata: Map<String, serde_json::Value>,
}

impl Entry {
    pub fn new(id: Uuid, players: Vec<Uuid>, metadata: Map<String, serde_json::Value>) -> Self {
        let timestamp = Utc::now();
        Self {
            id: EntryId(id),
            players,
            time_queued: timestamp,
            metadata,
        }
    }
}
