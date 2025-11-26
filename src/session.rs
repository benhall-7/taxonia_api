use redis::AsyncCommands;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::services::rand::generate_random_id;

#[derive(Clone)]
pub struct SessionStore {
    client: redis::Client,
}

#[derive(Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
}

impl SessionStore {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    fn session_key(id: &str) -> String {
        format!("session:{id}")
    }

    fn oauth_state_key(state: &str) -> String {
        format!("oauth_state:{state}")
    }

    pub async fn create_session(&self, user_id: i64) -> Result<String> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let session_id = generate_random_id();

        let data = SessionData {
            user_id,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&data)?;
        let key = Self::session_key(&session_id);

        // 7 days
        let _: () = conn.set_ex(key, json, 60 * 60 * 24 * 7).await?;

        Ok(session_id)
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<SessionData>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::session_key(id);
        let json: Option<String> = conn.get(key).await?;
        Ok(json.map(|j| serde_json::from_str(&j)).transpose()?)
    }

    pub async fn store_oauth_state(&self, state: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::oauth_state_key(state);
        // 10 minutes
        let _: () = conn.set_ex(key, "1", 600).await?;
        Ok(())
    }

    pub async fn consume_oauth_state(&self, state: &str) -> Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::oauth_state_key(state);
        let exists: bool = conn.exists(&key).await?;
        if exists {
            let _: () = conn.del(&key).await?;
        }
        Ok(exists)
    }
}
