use crate::services::inat::InatUser;
use crate::services::inat::TokenWithExpiry;
use reqwest::StatusCode;
use sqlx::{PgPool};

pub struct AuthRepo {
    pool: PgPool,
}

impl AuthRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_inat_user(
        &self,
        inat_user: &InatUser,
        token: &TokenWithExpiry,
    ) -> poem::Result<i64> {
        let inat_user_id = inat_user.id;
        let inat_login = inat_user.login.clone();
        let display_name = inat_user.name.clone().unwrap_or_else(|| inat_login.clone());
        // Find existing auth_identity
        // provider = 'inat'
        let provider = "inat";
        let provider_user_id = inat_user_id.to_string();

        #[derive(sqlx::FromRow)]
        struct ExistingIdentity {
            user_id: i64,
        }

        let mut tx =
            self.pool.begin().await.map_err(|_| {
                poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        
        // 1: check auth_identity
        let existing: Option<ExistingIdentity> = sqlx::query_as(
            r#"
            SELECT user_id
            FROM auth_identities
            WHERE provider = '$1' AND provider_user_id = $2
            "#,
        )
        .bind(&provider)
        .bind(&provider_user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

        // 2: update or insert users
        let user_id: i64 = if let Some(row) = existing {
            // update last_used_at
            sqlx::query(
                r#"
                UPDATE auth_identities
                SET last_used_at = now()
                WHERE provider = 'inat' AND provider_user_id = $1
                "#,
            )
            .bind(&provider_user_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

            // maybe update display_name on users
            sqlx::query(
                r#"
                UPDATE users
                SET display_name = $1, updated_at = now(), 
                    -- optionally: primary_email = ...
                    last_login_at = now()
                WHERE id = $2
                "#,
            )
            .bind(&display_name)
            .bind(row.user_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

            row.user_id
        } else {
            // insert new user
            let rec: (i64,) = sqlx::query_as(
                r#"
                INSERT INTO users (display_name, primary_email)
                VALUES ($1, NULL)
                RETURNING id
                "#,
            )
            .bind(&display_name)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| poem::Error::from_status(poem::http::StatusCode::INTERNAL_SERVER_ERROR))?;

            let new_user_id = rec.0;

            // insert auth_identity
            sqlx::query(
                r#"
                INSERT INTO auth_identities (
                    user_id, provider, provider_user_id, access_token, refresh_token, token_expires_at
                )
                VALUES (
                    $1, 'inat', $2, $3, $4, $5
                )
                "#,
            )
            .bind(new_user_id)
            .bind(&provider_user_id)
            .bind(&token.access_token)
            .bind(&token.refresh_token)
            .bind(token.expires_at)
            .execute(&mut *tx)
            .await.map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

            new_user_id
        };

        tx.commit()
            .await
            .map_err(|_| poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR))?;

        // 4: return user_id
        Ok(user_id)
    }
}
