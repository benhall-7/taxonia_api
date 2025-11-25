use std::env::var;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub app_env: AppEnv,
    pub bind_addr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    InvalidAppEnv(String),
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: var("DATABASE_URL")?,
            redis_url: var("REDIS_URL")?,
            app_env: var("APP_ENV")
                .map(AppEnv::try_from)
                .unwrap_or(Ok(AppEnv::Development))?,
            bind_addr: var("BIND_ADDR").unwrap_or("127.0.0.1:8080".into()),
        })
    }
}

impl TryFrom<String> for AppEnv {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "production" => Ok(AppEnv::Production),
            "development" => Ok(AppEnv::Development),
            _ => Err(ConfigError::InvalidAppEnv(format!(
                "Invalid APP_ENV: Received {value}. Expected \"production\" or \"development\""
            ))),
        }
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidAppEnv(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for ConfigError {}
