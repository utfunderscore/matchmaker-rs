use crate::algo::flexible::FlexibleMatchMaker;
use crate::queue::Entry;
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

pub trait Matchmaker: Send + Sync {

    fn get_type_name(&self) -> String;
    fn matchmake(&self, entries: Vec<Entry>) -> Result<Vec<Vec<Uuid>>, String>;

    fn is_valid_entry(&self, entry: &Entry) -> Result<(), String>;

    fn serialize(&self) -> Result<Value, String>;
}

pub type Deserializer = Box<dyn Fn(Value) -> Result<Box<dyn Matchmaker + Send + Sync>, String> + Send + Sync>;

lazy_static! {
    pub static ref DESERIALIZERS: HashMap<String, Deserializer> = {
        let mut m: HashMap<String, Deserializer> = HashMap::new();
        // Populate with actual deserializers
        m.insert("flexible".to_string(), Box::new(FlexibleMatchMaker::deserialize));
        m
    };
}

pub fn get_deserializer(
    type_name: &str,
) -> Option<&Deserializer> {
    DESERIALIZERS.get(type_name)
}