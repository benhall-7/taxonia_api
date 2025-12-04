use std::env::var;

#[derive(Debug, Clone)]
pub struct Config {
    pub allowed_origins: Vec<String>,
    pub database_url: String,
    pub redis_url: String,
    pub bind_addr: String,
    pub base_url: String,

    pub inat_client_id: String,
    pub inat_client_secret: String,
    pub inat_redirect_uri: String,
    pub inat_base_url: String,
    pub app_redirect_uri: String,

    // optional vars
    pub app_env: AppEnv,
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
            allowed_origins: std::env::var("ALLOWED_ORIGINS")?
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
            database_url: var("DATABASE_URL")?,
            redis_url: var("REDIS_URL")?,
            bind_addr: var("BIND_ADDR")?,
            base_url: var("BASE_URL")?,

            inat_client_id: var("INAT_CLIENT_ID")?,
            inat_client_secret: var("INAT_CLIENT_SECRET")?,
            inat_redirect_uri: var("INAT_REDIRECT_URI")?,
            inat_base_url: var("INAT_BASE_URL")?,
            app_redirect_uri: var("APP_REDIRECT_URI")?,

            app_env: var("APP_ENV")
                .map(AppEnv::try_from)
                .unwrap_or(Ok(AppEnv::Development))?,
        })
    }

    pub fn is_prod(&self) -> bool {
        self.app_env == AppEnv::Production
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
