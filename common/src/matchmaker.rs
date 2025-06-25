use crate::algo::flexible::FlexibleMatchMaker;
use crate::queue::Entry;
use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use uuid::Uuid;

pub enum MatchmakerResult {
    Matched(Vec<Vec<Uuid>>),
    Skip(String),
    Error((String, Option<Vec<Uuid>>)),
}

impl MatchmakerResult {
    
    pub fn is_matched(&self) -> bool {
        matches!(self, MatchmakerResult::Matched(_))
    }
    
    pub fn is_skip(&self) -> bool {
        matches!(self, MatchmakerResult::Skip(_))
    }
    
    pub fn is_err(&self) -> bool {
        matches!(self, MatchmakerResult::Error(_))
    }
    
    pub fn unwrap_skip(&self) -> String {
        if let MatchmakerResult::Skip(msg) = self {
            msg.clone()
        } else {
            panic!("Called unwrap_skip on a non-skip result");
        }
    }
    
    pub fn unwarp_err(&self) -> (String, Option<Vec<Uuid>>) {
        if let MatchmakerResult::Error(err) = self {
            err.clone()
        } else {
            panic!("Called unwarp_err on a non-error result");
        }
    }
    
}

pub trait Matchmaker: Send + Sync {
    fn get_type_name(&self) -> String;
    fn matchmake(
        &self,
        entries: Vec<&Entry>,
    ) -> MatchmakerResult;

    fn is_valid_entry(&self, entry: &Entry) -> Result<(), Box<dyn Error>>;

    fn serialize(&self) -> Result<Value, Box<dyn Error>>;
}

pub type Deserializer =
    Box<dyn Fn(Value) -> Result<Box<dyn Matchmaker + Send + Sync>, Box<dyn Error>> + Send + Sync>;

lazy_static! {
    pub static ref DESERIALIZERS: HashMap<String, Deserializer> = {
        let mut m: HashMap<String, Deserializer> = HashMap::new();
        // Populate with actual deserializers
        m.insert("flexible".to_string(), Box::new(FlexibleMatchMaker::deserialize));
        m
    };
}

pub fn serialize(matchmaker: &Box<dyn Matchmaker + Send + Sync>) -> Result<Value, Box<dyn Error>> {
    let json = serde_json::json!({
        "type": matchmaker.get_type_name(),
        "settings": matchmaker.serialize()?,
    });

    Ok(json)
}

pub fn deserialize(json: Value) -> Result<Box<dyn Matchmaker + Send + Sync>, Box<dyn Error>> {
    let type_name = json
        .get("type")
        .and_then(Value::as_str)
        .ok_or("Missing or invalid 'type' field in JSON")?
        .to_string();

    let deserializer = DESERIALIZERS
        .get(&type_name)
        .ok_or(format!("Unknown matchmaker type: {}", type_name))?;

    let settings = json
        .get("settings")
        .ok_or("Missing 'settings' field in JSON")?;

    deserializer(settings.to_owned())
}

pub fn get_deserializer(type_name: &str) -> Option<&Deserializer> {
    DESERIALIZERS.get(type_name)
}
