use dotenvy::dotenv;
use poem::{
    Route, Server, listener::TcpListener,
};
use poem_openapi::{OpenApiService, SecurityScheme, auth::Basic};
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::state::AppState;

pub mod config;
pub mod models;
pub mod routes;
pub mod state;

#[derive(SecurityScheme)]
#[oai(ty = "basic")]
struct MyBasicAuthorization(Basic);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // doesn't do anything in production, since env vars are included in the process
    let _ = dotenv();
    let config = Config::from_env()?;

    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    let redis_client = redis::Client::open(config.redis_url)?;

    let state = AppState::new(pool, redis_client);

    let api_service = OpenApiService::new(
        (
            routes::health_check::HealthCheckApi,
            routes::auth::AuthApi { state: state.clone() },
        ),
        "Taxonia API",
        "1.0",
    )
    .server("http://localhost:3000/api");

    // Swagger UI for testing & docs
    let swagger = api_service.swagger_ui();

    // Mount everything
    let api = Route::new().nest("/api", api_service).nest("/", swagger);

    // Start the server
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(api)
        .await?;

    Ok(())
}
