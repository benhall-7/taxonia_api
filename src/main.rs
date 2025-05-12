use axum::{Json, Router, routing::get};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use tokio::net::TcpListener;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    FmtSubscriber::builder().with_level(true).init();

    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;

    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(pool);

    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn health_check() -> Json<&'static str> {
    Json("OK")
}
