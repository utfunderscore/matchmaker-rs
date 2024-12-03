use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Game {
    server_host: String,
    server_port: u16,
    game_id: String,
}

impl Game {
    pub fn new(server_host: String, server_port: u16, game_id: String) -> Game {
        Game {
            server_host,
            server_port,
            game_id,
        }
    }
}
