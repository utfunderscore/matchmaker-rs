use jsonpath_rust::JsonPath;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::{fs, io};
use tokio::sync::RwLock;
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

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub game_id: String,
    pub host: String,
    pub port: u16,
}

impl Game {
    pub fn new(game_id: String, host: String, port: u16) -> Game {
        Game {
            game_id,
            host,
            port,
        }
    }
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
        GameFinderSettings {
            base_url: "http://example.com/{playlist}".into(),
            id_path: "$.gameId".to_string(),
            host_path: "$.host".to_string(),
            port_path: "$.port".to_string(),
        }
    }
}

impl GameFinderSettings {
    pub async fn load_or_create_config<P: AsRef<Path>>(
        path: P,
    ) -> Result<GameFinderSettings, GameFinderError> {
        if fs::metadata(&path).await.is_ok() {
            let data = fs::read_to_string(&path).await.map_err(GameFinderError::ConfigIo)?;
            let config: GameFinderSettings = serde_json::from_str(&data).map_err(GameFinderError::ConfigParse)?;
            Ok(config)
        } else {
            let config = GameFinderSettings::default();
            let data = serde_json::to_string_pretty(&config).map_err(GameFinderError::ConfigParse)?;
            fs::write(&path, data).await.map_err(GameFinderError::ConfigIo)?;
            Ok(config)
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameFinder {
    pub config: Arc<RwLock<GameFinderSettings>>,
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

impl GameFinder {
    pub fn new(config: Arc<RwLock<GameFinderSettings>>) -> GameFinder {
        GameFinder { config }
    }

    pub async fn find_game(
        &self,
        playlist: &str,
        players: &Vec<Vec<Uuid>>,
    ) -> Result<Game, GameFinderError> {
        let config = self.config.read().await;
        let url = config.base_url.replace("{playlist}", playlist);

        let response = CLIENT.get(&url).json(&players).send().await.map_err(GameFinderError::Http)?;

        if !response.status().is_success() {
            return Err(GameFinderError::GameNotFound(response.status()));
        }

        let body = response.json::<Value>().await.map_err(GameFinderError::Http)?;

        let game_id = body
            .query(&config.id_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_str())
            .ok_or(GameFinderError::InvalidField("gameId"))?;
        let host = body
            .query(&config.host_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_str())
            .ok_or(GameFinderError::InvalidField("host"))?;
        let port = body
            .query(&config.port_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_u64())
            .ok_or(GameFinderError::InvalidPort)?;

        Ok(Game::new(game_id.into(), host.into(), port as u16))
    }
}
