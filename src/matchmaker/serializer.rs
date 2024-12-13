use crate::matchmaker::implementations;
use crate::matchmaker::implementations::Matchmaker;
use serde_json::Value;
use std::collections::HashMap;

pub struct Registry {
    registry: HashMap<String, Box<dyn Serializer>>,
}

impl Registry {
    pub fn new() -> Self {
        let mut registry = Registry {
            registry: HashMap::new(),
        };

        registry.register(String::from("unrated"), Box::new(Unrated {}));

        registry
    }

    pub fn register(&mut self, namespace: String, serializer: Box<dyn Serializer>) {
        self.registry.insert(namespace, serializer);
    }

    #[allow(clippy::borrowed_box)]
    pub fn get(&self, namespace: &str) -> Option<&Box<dyn Serializer>> {
        self.registry.get(namespace).take()
    }
}

pub trait Serializer {
    #[allow(clippy::borrowed_box)]
    fn serialize(&self, matchmaker: &Box<dyn Matchmaker + Send + Sync>) -> Result<Value, String>;

    fn deserialize(&self, data: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, String>;
}

pub struct Unrated;

impl Serializer for Unrated {
    fn serialize(&self, matchmaker: &Box<dyn Matchmaker + Send + Sync>) -> Result<Value, String> {
        let matchmaker = matchmaker
            .as_any()
            .downcast_ref::<implementations::Unrated>()
            .ok_or("Invalid type provided for UnratedMatchmaker")?;

        serde_json::to_value(matchmaker)
            .map_err(|_| String::from("Invalid type provided for UnratedMatchmaker"))
    }

    fn deserialize(&self, data: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, String> {
        let matchmaker: implementations::Unrated =
            serde_json::from_value(data).map_err(|x| x.to_string())?;

        Ok(Box::new(matchmaker))
    }
}
