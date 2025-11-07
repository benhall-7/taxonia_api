use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
