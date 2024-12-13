use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Game {
    id: String,
    server_host: String,
    server_port: u16,
}

impl Game {
    pub fn new(server_host: String, server_port: u16, game_id: String) -> Game {
        Game {
            server_host,
            server_port,
            id: game_id,
        }
    }
}

pub trait GameProvider {
    fn get_game_server(&self) -> Result<Game, String>;
}

pub struct FakeGame;

impl GameProvider for FakeGame {
    fn get_game_server(&self) -> Result<Game, String> {
        let game = Game::new(String::from("localhost"), 25565, String::from("test"));

        Ok(game)
    }
}
