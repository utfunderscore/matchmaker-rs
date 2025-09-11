use crate::algo::elo::EloMatchmaker;
use crate::algo::flexible::FlexibleMatchMaker;
use crate::entry::{Entry, EntryId};
use serde_json::Value;
use std::error::Error;

#[derive(PartialEq)]
pub enum MatchmakerResult {
    Matched(Vec<Vec<EntryId>>),
    Skip(String),
    Error(String, Option<EntryId>),
}

impl MatchmakerResult {
    pub fn is_matched(&self) -> bool {
        matches!(self, MatchmakerResult::Matched(_))
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, MatchmakerResult::Skip(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, MatchmakerResult::Error(_, _))
    }

    pub fn unwrap_skip(&self) -> String {
        if let MatchmakerResult::Skip(msg) = self {
            msg.clone()
        } else {
            panic!("Called unwrap_skip on a non-skip result");
        }
    }

    pub fn unwarp_err(&self) -> String {
        if let MatchmakerResult::Error(err, affected) = self {
            err.clone()
        } else {
            panic!("Called unwarp_err on a non-error result");
        }
    }
}

pub trait Matchmaker: Send + Sync {
    fn get_type_name(&self) -> String;
    fn matchmake(&self) -> MatchmakerResult;

    fn serialize(&self) -> Result<Value, Box<dyn Error>>;

    fn remove_all(&mut self) -> Vec<Entry>;

    fn get_entries(&self) -> Vec<&Entry>;

    fn remove_entry(&mut self, entry_id: &EntryId) -> Result<Entry, Box<dyn Error>>;

    fn get_entry(&self, entry_id: &EntryId) -> Option<&Entry>;

    fn add_entry(&mut self, entry: Entry) -> Result<(), Box<dyn Error>>;
}

pub fn deserialize(
    name: String,
    value: Value,
) -> Result<Box<dyn Matchmaker + Send + Sync>, Box<dyn Error>> {
    match name.as_str() {
        "elo" => EloMatchmaker::deserialize(value),
        "flexible" => FlexibleMatchMaker::deserialize(value),
        _ => Err(format!("Unknown matchmaker type: {}", name).into()),
    }
}
