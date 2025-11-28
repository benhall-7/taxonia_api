use crate::{
    internal_error,
    services::{self, auth::UserRow},
};
use poem::{
    http::StatusCode,
    web::cookie::{Cookie, CookieJar, SameSite},
};
use poem_openapi::{
    Object, OpenApi,
    payload::{self, Json},
};
use rand::{Rng, distr::Alphanumeric};
use serde::Serialize;

use crate::clients::inat::InatClient;
use crate::repos::user_repo::UserRepo;
use crate::session_store::SessionStore;
use crate::state::AppState;

pub struct AuthApi {
    pub state: AppState,
}

#[OpenApi(prefix_path = "/auth")]
impl AuthApi {
    #[oai(path = "/login-url", method = "get")]
    async fn login_url(&self) -> poem::Result<Json<LoginUrlResponse>> {
        let session_repo = SessionStore::new(self.state.redis.clone());
        let cfg = &self.state.config;

        // Generate random state
        let state: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Store state in Redis with short TTL (e.g. 10 minutes)
        session_repo
            .store_oauth_state(&state)
            .await
            .map_err(|e| internal_error("store_oauth_state failed", e))?;

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
        let session_repo = SessionStore::new(self.state.redis.clone());

        // 1: Validate state (consume only)
        let exists = session_repo
            .consume_oauth_state(&state)
            .await
            .map_err(|e| internal_error("consume_oauth_state failed", e))?;
        if !exists {
            return Err(poem::Error::from_status(StatusCode::BAD_REQUEST));
        }

        // 2: get OAuth token
        let inat_client = InatClient::new();
        let token_with_exp = inat_client
            .exchange_code_for_token(&self.state.config, &code)
            .await
            .map_err(|e| internal_error("exchange_code_for_token failed", e))?;

        // 3: get JWT api_token
        let api_token = inat_client
            .exchange_access_for_api_token(&self.state.config, &token_with_exp.access_token)
            .await
            .map_err(|e| internal_error("exchange_access_for_api_token failed", e))?;

        // 4: get iNat user profile
        let inat_user = inat_client
            .fetch_current_user(&api_token)
            .await
            .map_err(|e| internal_error("fetch_current_user failed", e))?;

        // 5: Upsert user + auth_identity
        let auth = UserRepo::new(self.state.db.clone());
        let user_id = auth
            .upsert_inat_user(&inat_user, &token_with_exp)
            .await
            .map_err(|e| internal_error("upsert_inat_user failed", e))?;

        // 6: Create session in Redis
        let session_id = session_repo
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
    async fn me(&self, jar: &CookieJar) -> poem::Result<Json<MeResponse>> {
        let user = services::auth::get_current_user(&self.state, jar).await?;
        Ok(Json(MeResponse::from(user)))
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
    primary_email: Option<String>,
}

impl From<UserRow> for MeResponse {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            primary_email: value.primary_email,
        }
    }
}
