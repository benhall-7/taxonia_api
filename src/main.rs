use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, AuthSession};
use dotenvy::dotenv;
use serde::Deserialize;
use sqlx::PgPool;
use std::{env, sync::Arc};
use tokio::{net::TcpListener, signal, task::AbortHandle};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::FmtSubscriber;

use crate::user::Backend;

pub mod user;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    FmtSubscriber::builder().with_level(true).init();

    // TODO: clean up state into one place
    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;

    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store);
    let backend = Backend::new(pool);
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer.clone()).build();

    let router = Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/health_check", get(health_check))
        .with_state(backend)
        .layer(auth_layer);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn health_check() -> Json<&'static str> {
    Json("OK")
}

// TODO: reorganize routes and their parameters
#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

async fn login(
    mut auth_session: AuthSession<Backend>,
    Json(creds): Json<LoginForm>,
) -> impl IntoResponse {
    let user = auth_session
        .authenticate((creds.email, creds.password))
        .await
        .unwrap_or(None);

    if let Some(user) = user {
        auth_session.login(&user).await.unwrap();
        Json(serde_json::json!({ "message": "logged in" })).into_response()
    } else {
        (axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }
}

async fn me(auth_session: AuthSession<Backend>) -> impl IntoResponse {
    if let Some(user) = auth_session.user {
        Json(serde_json::json!({ "email": user.email })).into_response()
    } else {
        (axum::http::StatusCode::UNAUTHORIZED, "Not logged in").into_response()
    }
}
