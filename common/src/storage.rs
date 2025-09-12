use sea_orm::Database;

struct Storage {
}

impl Storage {

    pub async fn new(url: String) {

        let db = Database::connect(url).await;

    }

}