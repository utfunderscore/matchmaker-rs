use crate::queue::Entry;

pub struct GameResult {
    pub address: String,
    pub access_token: String,
}

pub trait ServerFinder {
    async fn find_servers(
        &self,
        playlist: &str,
        teams: Vec<Vec<Entry>>,
    ) -> Result<Vec<String>, String>;
}
