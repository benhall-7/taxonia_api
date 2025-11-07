pub struct Config {
    pub database_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL missing"),
            redis_url: std::env::var("REDIS_URL").expect("REDIS_URL missing"),
        }
    }
}
