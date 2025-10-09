use axum_login::{AuthUser, AuthnBackend, UserId};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    // Used to determine if a session has been invalidated
    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash
            .as_ref()
            .map(|hash| hash.as_bytes())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    pool: PgPool,
}

impl Backend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = (String, String); // email, password
    type Error = AuthError;

    async fn authenticate(
        &self,
        (email, password): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, password_hash FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(u) => match &u.password_hash {
                Some(hash) if verify_password(&password, &hash) => Ok(Some(u)),
                // password incorrect
                _ => Err(AuthError::InvalidCredentials),
            },
            // no user found
            None => Err(AuthError::InvalidCredentials),
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, email, password_hash FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}

fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}
