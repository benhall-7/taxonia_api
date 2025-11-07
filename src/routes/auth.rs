use argon2::{Argon2, PasswordHash, PasswordVerifier};
use poem::http::StatusCode;
use poem_openapi::{Object, OpenApi, payload::Json};
use sqlx::query_as;

use crate::db::Db;
use crate::models::User;

pub struct AuthApi {
    pub db: Db,
}

#[derive(Object)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[OpenApi]
impl AuthApi {
    /// Login using email and password
    #[oai(path = "/login", method = "post", operation_id = "login")]
    async fn login(
        &self,
        Json(payload): Json<LoginRequest>,
    ) -> poem::Result<Json<LoginResponse>> {
        let user = query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&self.db)
            .await?;

        let Some(user) = user else {
            return Err(poem::Error::from_status(StatusCode::UNAUTHORIZED));
        };

        if let Some(ref hash) = user.password_hash {
            let parsed = PasswordHash::new(hash)
                .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;
            if Argon2::default()
                .verify_password(payload.password.as_bytes(), &parsed)
                .is_ok()
            {
                return Ok(Json(LoginResponse {
                    message: format!("Welcome, {}", user.email),
                }));
            }
        }

        Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
    }
}

#[derive(Object)]
pub struct LoginResponse {
    message: String,
}
