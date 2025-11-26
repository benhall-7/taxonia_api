use poem::{
    http::StatusCode,
    web::cookie::{Cookie, CookieJar, SameSite},
};
use poem_openapi::{
    Object, OpenApi,
    payload::{self, Json},
};
use rand::{Rng, distr::Alphanumeric};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    auth_repo::AuthRepo,
    services::inat::{exchange_access_for_api_token, exchange_code_for_token, fetch_current_user},
    session::SessionStore,
    state::AppState,
};

fn internal_error<E: std::fmt::Display>(context: &'static str, err: E) -> poem::Error {
    error!("{context}: {err}");
    poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
}

pub struct AuthApi {
    pub state: AppState,
}

#[OpenApi(prefix_path = "/auth")]
impl AuthApi {
    #[oai(path = "/login-url", method = "get")]
    async fn login_url(&self) -> poem::Result<Json<LoginUrlResponse>> {
        // Generate random state
        let state: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store state in Redis with short TTL (e.g. 10 minutes)
        let mut conn = self
            .state
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        let key = format!("oauth_state:{}", state);
        let _: () = conn
            .set_ex(key, "1", 600)
            .await
            .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        let cfg = &self.state.config;
        let redirect = urlencoding::encode(&cfg.inat_redirect_uri);

        let url = format!(
            "{base}/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&response_type=code&scope=read",
            base = cfg.inat_base_url,
            client_id = cfg.inat_client_id,
            redirect_uri = redirect,
        );

        Ok(Json(LoginUrlResponse { url }))
    }

    /// iNaturalist OAuth callback
    #[oai(path = "/callback", method = "get")]
    async fn callback(
        &self,
        jar: &CookieJar,
        code: String,
        state: String,
    ) -> poem::Result<payload::Response<()>> {
        let cfg = &self.state.config;

        let session_store = SessionStore::new(self.state.redis.clone());

        // 1: Validate state (consume only)
        let exists = session_store
            .consume_oauth_state(&state)
            .await
            .map_err(|e| internal_error("consume_oauth_state failed", e))?;
        if !exists {
            return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
        }

        // 2: code -> OAuth token
        let token_with_exp = exchange_code_for_token(&self.state.config, &code)
            .await
            .map_err(|e| internal_error("exchange_code_for_token failed", e))?;

        // 3: OAuth token -> JWT api_token
        let api_token =
            exchange_access_for_api_token(&self.state.config, &token_with_exp.access_token)
                .await
                .map_err(|e| internal_error("exchange_access_for_api_token failed", e))?;

        // 4: JWT -> iNat user profile
        let inat_user = fetch_current_user(&api_token)
            .await
            .map_err(|e| internal_error("fetch_current_user failed", e))?;

        // 5: Upsert user + auth_identity
        let auth = AuthRepo::new(self.state.db.clone());
        let user_id = auth
            .upsert_inat_user(&inat_user, &token_with_exp)
            .await
            .map_err(|e| internal_error("upsert_inat_user failed", e))?;

        // 6: Create session in Redis
        let session_id = session_store
            .create_session(user_id)
            .await
            .map_err(|e| internal_error("create_session failed", e))?;

        // 7: Set cookie and redirect to frontend
        let mut cookie = Cookie::new("taxonia_session", session_id);
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie.set_secure(cfg.is_prod());
        cookie.set_path("/");

        jar.add(cookie);

        let resp = payload::Response::new(())
            .status(StatusCode::FOUND)
            .header("Location", "https://taxonia.app/auth/callback");

        Ok(resp)
    }

    /// Get current logged-in user
    #[oai(path = "/auth/me", method = "get")]
    async fn me(&self, req: &poem::Request) -> poem::Result<Json<MeResponse>> {
        // 1: Get session cookie
        let cookie = req
            .cookie()
            .get("taxonia_session")
            .ok_or_else(|| poem::Error::from_status(poem::http::StatusCode::UNAUTHORIZED))?;

        let session_id: String = cookie
            .value()
            .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
        let session_key = format!("session:{}", session_id);

        // 2: Fetch session from Redis
        let mut conn = self
            .state
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

        let session_json: Option<String> = conn
            .get(&session_key)
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;
        let session_json = match session_json {
            Some(s) => s,
            None => {
                return Err(poem::Error::from_status(
                    poem::http::StatusCode::UNAUTHORIZED,
                ));
            }
        };

        #[derive(Deserialize)]
        struct SessionData {
            user_id: i64,
        }

        let session: SessionData = serde_json::from_str(&session_json)
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::UNAUTHORIZED))?;

        // 3: Fetch user from DB (and any needed iNat info)
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: i64,
            display_name: String,
        }

        let user: UserRow = sqlx::query_as(r#"SELECT id, display_name FROM users WHERE id = $1"#)
            .bind(session.user_id)
            .fetch_one(&self.state.db)
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

        // For now, use display_name as inat_username (or extend schema later)
        Ok(Json(MeResponse {
            id: user.id,
            display_name: user.display_name.clone(),
            inat_username: user.display_name, // adjust once you store login separately
        }))
    }
}

#[derive(Object, Serialize)]
struct LoginUrlResponse {
    url: String,
}

#[derive(Object, Serialize)]
struct MeResponse {
    id: i64,
    display_name: String,
    inat_username: String,
}
