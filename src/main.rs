use dotenvy::dotenv;
use poem::{
    Error, IntoResponse, Result, Route, Server, http::StatusCode, listener::TcpListener, web::Json,
};
use poem_openapi::{OpenApi, OpenApiService, SecurityScheme, auth::Basic, payload::PlainText};
use serde::Deserialize;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::{signal, task::AbortHandle};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::FmtSubscriber;

use crate::routes::auth::LoginRequest;

pub mod app_error;
pub mod config;
pub mod db;
pub mod models;
pub mod routes;

#[derive(SecurityScheme)]
#[oai(ty = "basic")]
struct MyBasicAuthorization(Basic);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Initialize database connection
    let pool = db::init_pool().await?;

    // Build OpenAPI services
    let api_service = OpenApiService::new(
        (
            routes::health_check::HealthCheckApi,
            routes::auth::AuthApi { db: pool.clone() },
        ),
        "Taxonia API",
        "1.0",
    )
    .server("http://localhost:3000/api");

    // Swagger UI for testing & docs
    let swagger = api_service.swagger_ui();

    // Mount everything
    let app = Route::new().nest("/api", api_service).nest("/", swagger);

    // Start the server
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await?;

    Ok(())
}

// // TODO: reorganize routes and their parameters
// #[derive(Deserialize)]
// struct LoginForm {
//     email: String,
//     password: String,
// }

// async fn login(
//     mut auth_session: AuthSession<Backend>,
//     Json(creds): Json<LoginForm>,
// ) -> impl IntoResponse {
//     let user = auth_session
//         .authenticate((creds.email, creds.password))
//         .await
//         .unwrap_or(None);

//     if let Some(user) = user {
//         auth_session.login(&user).await.unwrap();
//         Json(serde_json::json!({ "message": "logged in" })).into_response()
//     } else {
//         (axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
//     }
// }

// async fn me(auth_session: AuthSession<Backend>) -> impl IntoResponse {
//     if let Some(user) = auth_session.user {
//         Json(serde_json::json!({ "email": user.email })).into_response()
//     } else {
//         (axum::http::StatusCode::UNAUTHORIZED, "Not logged in").into_response()
//     }
// }
