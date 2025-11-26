use poem_openapi::{Object, OpenApi};
use poem_openapi::payload::Json;
use redis::AsyncCommands;
use serde::Serialize;
use tokio::join;

use crate::state::AppState;

pub struct HealthCheckApi {
    pub state: AppState,
}

#[derive(Debug, Clone, Copy, Serialize, Object)]
pub struct HealthCheckResponse {
    db_ok: bool,
    redis_ok: bool,
    server_ok: bool,
}

#[OpenApi]
impl HealthCheckApi {
    /// Health check endpoint
    #[oai(path = "/health_check", method = "get", operation_id = "health_check")]
    async fn health_check(&self) -> poem::Result<Json<HealthCheckResponse>> {
        // TODO: Add timeout handling for the DB and Redis checks independently
        let db_check = sqlx::query("SELECT 1").execute(&self.state.db);

        let redis_check = async {
            let mut conn = self.state.redis.get_multiplexed_async_connection().await?;
            let _ = conn.ping::<String>().await?;
            Ok::<(), redis::RedisError>(())
        };

        let (db_res, redis_res) = join!(db_check, redis_check);
        let (db_ok, redis_ok) = (db_res.is_ok(), redis_res.is_ok());

        Ok(Json(HealthCheckResponse {
            db_ok,
            redis_ok,
            server_ok: true,
        }))
    }
}
