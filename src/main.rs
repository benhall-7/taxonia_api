use dotenvy::dotenv;
use poem::EndpointExt;
use poem::middleware::{CookieJarManager, Cors};
use poem::{Route, Server, listener::TcpListener};
use poem_openapi::OpenApiService;
use reqwest::StatusCode;
use sqlx::postgres::PgPoolOptions;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;

use crate::config::{AppEnv, Config};
use crate::state::AppState;

pub mod clients;
pub mod config;
pub mod models;
pub mod repos;
pub mod routes;
pub mod services;
pub mod session_store;
pub mod state;

pub fn internal_error<E: std::fmt::Display>(context: &'static str, err: E) -> poem::Error {
    error!("{context}: {err}");
    poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // doesn't do anything in production, since env vars are included in the process
    let _ = dotenv();
    let config = Config::from_env()?;
    // just clone first to avoid borrow issues
    let state_config = config.clone();

    init_tracing(&config);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    let redis_client = redis::Client::open(config.redis_url)?;

    let state = AppState::new(pool, redis_client, state_config);

    let cors = Cors::new()
        .allow_credentials(true)
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allow_headers(vec!["Content-Type", "Authorization"])
        .allow_origins(config.allowed_origins);

    let api_service = OpenApiService::new(
        (
            routes::health_check::HealthCheckApi {
                state: state.clone(),
            },
            routes::auth::AuthApi {
                state: state.clone(),
            },
            routes::quiz::QuizApi {
                state: state.clone(),
            },
        ),
        "Taxonia API",
        "1.0",
    )
    .server(format!("{}", config.bind_addr));

    // Swagger UI for testing & docs
    let swagger = api_service.swagger_ui();
    let spec = api_service.spec_endpoint();

    // Mount everything
    let api = Route::new()
        .nest("/", api_service)
        .nest("/spec", swagger)
        .nest("/spec.json", spec)
        .with(CookieJarManager::new())
        .with(cors);

    Server::new(TcpListener::bind(config.bind_addr))
        .run(api)
        .await?;

    Ok(())
}

fn init_tracing(config: &Config) {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("sqlx=info".parse().unwrap())
        .add_directive("taxonia_api=debug".parse().unwrap());

    if config.app_env == AppEnv::Production {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .with_current_span(false)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .pretty()
            .init();
    }
}
