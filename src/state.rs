use redis::Client;
use sqlx::{Pool, Postgres};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub redis: Client,
    pub config: Config,
}

impl AppState {
    pub fn new(db: Pool<Postgres>, redis: Client, config: Config) -> Self {
        Self { db, redis, config }
    }
}
