use redis::Client;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub redis: Client,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, redis: Client) -> Self {
        Self { db, redis }
    }
}
