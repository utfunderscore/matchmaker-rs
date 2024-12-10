use crate::matchmaker::matchmaker::{Matchmaker, UnratedMatchmaker};
use serde_json::Value;
use std::collections::HashMap;

pub struct SerializerRegistry {
    registry: HashMap<String, Box<dyn Serializer>>,
}

impl SerializerRegistry {
    pub fn new() -> Self {
        let mut registry = SerializerRegistry {
            registry: HashMap::new(),
        };

        registry.register(String::from("unrated"), Box::new(UnratedSerializer {}));

        registry
    }

    pub fn register(&mut self, namespace: String, serializer: Box<dyn Serializer>) {
        self.registry.insert(namespace, serializer);
    }

    pub fn get(&self, namespace: String) -> Option<&Box<dyn Serializer>> {
        self.registry.get(&namespace).take()
    }
}

pub trait Serializer {
    fn serialize(&self, matchmaker: &Box<dyn Matchmaker + Send + Sync>) -> Result<Value, String>;

    fn deserialize(&self, data: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, String>;
}

pub struct UnratedSerializer;

impl Serializer for UnratedSerializer {
    fn serialize(&self, matchmaker: &Box<dyn Matchmaker + Send + Sync>) -> Result<Value, String> {
        let matchmaker = matchmaker
            .as_any()
            .downcast_ref::<UnratedMatchmaker>()
            .ok_or("Invalid type provided for UnratedMatchmaker")?;

        serde_json::to_value(matchmaker)
            .map_err(|_| String::from("Invalid type provided for UnratedMatchmaker"))
    }

    fn deserialize(&self, data: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, String> {
        let matchmaker: UnratedMatchmaker =
            serde_json::from_value(data).map_err(|x| x.to_string())?;

        Ok(Box::new(matchmaker))
    }
}
