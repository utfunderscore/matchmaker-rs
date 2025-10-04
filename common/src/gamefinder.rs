use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::io;
use tracing::info;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum GameFinderError {
    #[error("Failed to read or write config file: {0}")]
    ConfigIo(#[from] io::Error),
    #[error("Failed to parse config file: {0}")]
    ConfigParse(#[from] serde_json::Error),
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Game not found (HTTP status: {0})")]
    GameNotFound(reqwest::StatusCode),
    #[error("Missing or invalid field in response: {0}")]
    InvalidField(&'static str),
    #[error("Port value is invalid or missing")]
    InvalidPort,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameFinderSettings {
    pub base_url: String,
    pub id_path: String,
    pub host_path: String,
    pub port_path: String,
}

impl Default for GameFinderSettings {
    fn default() -> GameFinderSettings {
        let settings = GameFinderSettings {
            base_url: std::env::var("GAMEFINDER_BASE_URL")
                .unwrap_or_else(|_| "http://example.com/{playlist}".into()),
            id_path: std::env::var("GAMEFINDER_ID_PATH").unwrap_or_else(|_| "$.gameId".to_string()),
            host_path: std::env::var("GAMEFINDER_HOST_PATH")
                .unwrap_or_else(|_| "$.host".to_string()),
            port_path: std::env::var("GAMEFINDER_PORT_PATH")
                .unwrap_or_else(|_| "$.port".to_string()),
        };

        info!("Game finder settings: {:?}", settings);

        settings
    }
}

#[derive(Debug, Clone)]
pub struct GameFinder {
    pub config: GameFinderSettings,
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

impl GameFinder {
    pub fn new() -> GameFinder {
        GameFinder {
            config: GameFinderSettings::default(),
        }
    }

    pub async fn find_game(
        &self,
        playlist: &str,
        players: &Vec<Vec<Uuid>>,
    ) -> Result<Value, GameFinderError> {
        let url = self.config.base_url.replace("{playlist}", playlist);

        info!("Making game request to {}", url);

        let response = CLIENT
            .get(&url)
            .json(&players)
            .send()
            .await
            .map_err(GameFinderError::Http)?;

        if !response.status().is_success() {
            return Err(GameFinderError::GameNotFound(response.status()));
        }
        response
            .json::<Value>()
            .await
            .map_err(GameFinderError::Http)
    }
}
