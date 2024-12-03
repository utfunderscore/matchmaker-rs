use crate::game::game::Game;

pub trait GameProvider {
    fn get_game_server(&self) -> Result<Game, String>;
}

pub struct FakeGameProvider;

impl GameProvider for FakeGameProvider {
    fn get_game_server(&self) -> Result<Game, String> {
        let game = Game::new(String::from("localhost"), 25565, String::from("test"));

        Ok(game)
    }
}
