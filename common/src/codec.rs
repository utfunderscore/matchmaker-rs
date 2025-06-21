use crate::registry::ThreadMatchmaker;
use serde_json::Value;
use std::collections::HashMap;

pub type MatchmakerDeserializer = fn(Value) -> Result<Box<ThreadMatchmaker>, String>;
pub type MatchmakerSerializer = fn(&Box<ThreadMatchmaker>) -> Result<Value, String>;

#[derive(Clone)]
pub struct Codec {
    deserializers: HashMap<String, Box<MatchmakerDeserializer>>,
    serializers: HashMap<String, Box<MatchmakerSerializer>>,
}

impl Codec {
    pub fn new() -> Codec {
        Codec {
            deserializers: HashMap::new(),
            serializers: HashMap::new(),
        }
    }

    pub fn register_deserializer(&mut self, name: &str, constructor: MatchmakerDeserializer) {
        self.deserializers.insert(name.to_lowercase(), Box::new(constructor));
    }

    pub fn get_deserializer(&self, name: &str) -> Option<&Box<MatchmakerDeserializer>> {
        self.deserializers.get(&name.to_lowercase())
    }

    pub fn register_serializer(&mut self, name: &str, constructor: Box<MatchmakerSerializer>) {
        self.serializers.insert(name.to_lowercase(), constructor);
    }

    pub fn get_serializer(&self, name: &str) -> Option<&Box<MatchmakerSerializer>> {
        self.serializers.get(&name.to_lowercase())
    }
}
