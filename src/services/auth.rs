use poem::Error as PoemError;
use poem::Result as PoemResult;
use poem::http::StatusCode;
use poem::web::cookie::CookieJar;
use sqlx::FromRow;

use crate::internal_error;
use crate::session_store::SessionStore;
use crate::state::AppState; // the helper we defined earlier

#[derive(FromRow)]
pub struct UserRow {
    pub id: i64,
    pub display_name: String,
    pub primary_email: Option<String>,
}

// Helper: get current user row or return 401/500
pub async fn get_current_user(state: &AppState, jar: &CookieJar) -> PoemResult<UserRow> {
    // 1) Read session cookie
    let cookie = jar
        .get("taxonia_session")
        .ok_or_else(|| PoemError::from_status(StatusCode::UNAUTHORIZED))?;

    let session_id: String = cookie
        .value()
        .map_err(|_| PoemError::from_status(StatusCode::UNAUTHORIZED))?;

    // 2) Resolve session via Redis
    let session_store = SessionStore::new(state.redis.clone());
    let session = session_store
        .get_session(&session_id)
        .await
        .map_err(|e| internal_error("get_session failed", e))?;

    let session = match session {
        Some(s) => s,
        None => return Err(PoemError::from_status(StatusCode::UNAUTHORIZED)),
    };

    // 3) Fetch user row from DB
    let user: UserRow = sqlx::query_as(
        r#"
        SELECT id, display_name, primary_email
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(session.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| internal_error("fetch_current_user_row failed", e))?;

    Ok(user)
}
