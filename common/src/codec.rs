use crate::registry::ThreadMatchmaker;
use serde_json::Value;
use std::collections::HashMap;

pub type MatchmakerDeserializer = fn(Value) -> Result<Box<ThreadMatchmaker>, String>;
pub type MatchmakerSerializer = fn(&Box<ThreadMatchmaker>) -> Result<Value, String>;

#[derive(Clone)]
pub struct Codec {
    deserializers: HashMap<String, MatchmakerDeserializer>,
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec {
    pub fn new() -> Codec {
        Codec {
            deserializers: HashMap::new(),
        }
    }

    pub fn register_deserializer(&mut self, name: &str, constructor: MatchmakerDeserializer) {
        self.deserializers
            .insert(name.to_lowercase(), constructor);
    }

    pub fn get_deserializer(&self, name: &str) -> Option<&MatchmakerDeserializer> {
        self.deserializers.get(&name.to_lowercase())
    }
}
