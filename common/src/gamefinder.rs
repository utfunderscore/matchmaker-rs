use jsonpath_rust::JsonPath;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::path::Path;
use tokio::{fs, io};
use uuid::Uuid;

pub struct GameResult {
    pub game_id: String,
    pub host: String,
    pub port: u16,
}

impl GameResult {
    pub fn new(game_id: String, host: String, port: u16) -> GameResult {
        GameResult {
            game_id,
            host,
            port,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GameFinderConfig {
    pub base_url: String,
    pub id_path: String,
    pub host_path: String,
    pub port_path: String,
}

impl Default for GameFinderConfig {
    fn default() -> GameFinderConfig {
        GameFinderConfig {
            base_url: "http://example.com/{playlist}".into(),
            id_path: "$.gameId".to_string(),
            host_path: "$.host".to_string(),
            port_path: "$.port".to_string(),
        }
    }
}

impl GameFinderConfig {
    pub async fn load_or_create_config<P: AsRef<Path>>(
        path: P,
    ) -> Result<GameFinderConfig, Box<dyn Error>> {
        if path.as_ref().exists() {
            let config_str: String = fs::read_to_string(&path).await?;
            let config: GameFinderConfig = toml::from_str(&config_str)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(config)
        } else {
            let config = GameFinderConfig::default();
            let toml_str = toml::to_string_pretty(&config).unwrap();
            fs::write(&path, toml_str).await?;
            Ok(config)
        }
    }
}

pub struct GameFinder {
    pub client: Client,
    pub config: GameFinderConfig,
}

impl GameFinder {

    pub fn new(config: GameFinderConfig) -> GameFinder {
        let client = Client::new();
        GameFinder { client, config }
    }

    pub async fn find_game(
        &self,
        playlist: String,
        players: Vec<Vec<Uuid>>,
    ) -> Result<GameResult, Box<dyn Error>> {
        let url = self.config.base_url.replace("{playlist}}", playlist.as_str());

        let response = self.client.get(&url).json(&players).send().await?;

        if !response.status().is_success() {
            return Err("Game not found".into());
        }

        let body = response.json::<Value>().await?;

        let game_id = body
            .query(&self.config.id_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let host = body
            .query(&self.config.host_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let port = body
            .query(&self.config.port_path)
            .unwrap_or_default()
            .first()
            .and_then(|x| x.as_u64())
            .unwrap_or(0);

        Ok(GameResult::new(game_id.into(), host.into(), port as u16))
    }
}
