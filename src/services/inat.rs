use crate::config::Config;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;

static INAT_API_BASE: &str = "https://api.inaturalist.org/v1";

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>, // seconds
    pub created_at: Option<i64>, // unix timestamp (seconds)
}

#[derive(Debug)]
pub struct TokenWithExpiry {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

// 1) code -> OAuth access token
pub async fn exchange_code_for_token(cfg: &Config, code: &str) -> Result<TokenWithExpiry> {
    let client = Client::new();
    let url = format!("{}/oauth/token", cfg.inat_base_url);

    let params = [
        ("client_id", cfg.inat_client_id.as_str()),
        ("client_secret", cfg.inat_client_secret.as_str()),
        ("code", code),
        ("redirect_uri", cfg.inat_redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];

    let resp = client
        .post(url)
        .form(&params) // x-www-form-urlencoded
        .send()
        .await?
        .error_for_status()?;

    let body: OAuthTokenResponse = resp.json().await?;

    let expires_at = match (body.created_at, body.expires_in) {
        (Some(created), Some(expires_in)) => Some(
            DateTime::<Utc>::from_timestamp(created, 0).unwrap_or_else(|| Utc::now())
                + Duration::seconds(expires_in),
        ),
        _ => None,
    };

    Ok(TokenWithExpiry {
        access_token: body.access_token,
        refresh_token: body.refresh_token,
        expires_at,
    })
}

#[derive(Debug, Deserialize)]
struct ApiTokenResponse {
    api_token: String,
}

// 2) OAuth access token -> JWT api_token
pub async fn exchange_access_for_api_token(cfg: &Config, access_token: &str) -> Result<String> {
    let client = Client::new();
    let url = format!("{}/users/api_token", cfg.inat_base_url);

    let resp = client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?;

    let body: ApiTokenResponse = resp.json().await?;
    Ok(body.api_token)
}

#[derive(Debug, Deserialize, Clone)]
pub struct InatUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsersMeResponse {
    results: Vec<InatUser>,
}

// 3) JWT api_token -> user info
pub async fn fetch_current_user(api_token: &str) -> Result<InatUser> {
    let client = Client::new();
    let url = format!("{}/users/me", INAT_API_BASE);

    let resp = client
        .get(url)
        .bearer_auth(api_token)
        .send()
        .await?
        .error_for_status()?;

    let body: UsersMeResponse = resp.json().await?;
    let user = body
        .results
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("empty results from /v1/users/me"))?;

    Ok(user)
}
