use chrono::{DateTime, Utc};
use poem::web::cookie::CookieJar;
use poem_openapi::{Object, OpenApi, param::Query, payload::Json};
use serde_json::Value;

use crate::internal_error;
use crate::repos::quiz_repo::{QuizRepo, QuizResultRow};
use crate::services::auth::get_current_user;
use crate::state::AppState;

#[derive(Clone)]
pub struct QuizApi {
    pub state: AppState,
}

#[derive(Object, Debug)]
struct SaveQuizResultRequest {
    /// A string identifier for the quiz mode, e.g. "inat_observations"
    quiz_type: String,
    /// Arbitrary quiz configuration (taxon filters, place, etc.)
    params: Value,
    /// Final score from 0.0 to 1.0
    score: f64,
    /// Optional total number of questions
    question_count: Option<i32>,
    /// Optional duration in seconds
    duration_seconds: Option<i32>,
}

#[derive(Object, Debug)]
struct SaveQuizResultResponse {
    id: i64,
}

#[derive(Object, Debug)]
struct QuizResultResponse {
    id: i64,
    quiz_type: String,
    params: Value,
    score: f64,
    question_count: Option<i32>,
    duration_seconds: Option<i32>,
    created_at: DateTime<Utc>,
}

#[derive(Object)]
struct ListQuizResultsResponse {
    items: Vec<QuizResultResponse>,
}

#[OpenApi(prefix_path = "/quiz")]
impl QuizApi {
    // /// Save a completed quiz result for the current user
    #[oai(path = "/results", method = "post")]
    async fn save_result(
        &self,
        jar: &CookieJar,
        Json(body): Json<SaveQuizResultRequest>,
    ) -> poem::Result<Json<SaveQuizResultResponse>> {
        // Require login
        let user = get_current_user(&self.state, jar).await?;

        let repo = QuizRepo::new(self.state.db.clone());

        // Clamp score to [0,1] just in case
        let score = body.score.clamp(0.0, 1.0);

        let id = repo
            .insert_quiz_result(
                user.id,
                &body.quiz_type,
                &body.params,
                score,
                body.question_count,
                body.duration_seconds,
            )
            .await
            .map_err(|e| internal_error("insert_quiz_result failed", e))?;

        Ok(Json(SaveQuizResultResponse { id }))
    }

    /// List recent quiz results for the current user
    #[oai(path = "/results", method = "get")]
    async fn list_results(
        &self,
        jar: &CookieJar,
        #[oai(default = "default_limit")] limit: Query<i64>,
        #[oai(default = "default_offset")] offset: Query<i64>,
    ) -> poem::Result<Json<ListQuizResultsResponse>> {
        let limit = limit.0.clamp(1, 100);
        let offset = offset.0.max(0);
        let user = get_current_user(&self.state, jar).await?;

        let repo = QuizRepo::new(self.state.db.clone());
        let rows = repo
            .list_quiz_results_for_user(user.id, limit, offset)
            .await
            .map_err(|e| internal_error("list_quiz_results_for_user failed", e))?;

        let items = rows
            .into_iter()
            .map(|r| QuizResultResponse {
                id: r.id,
                quiz_type: r.quiz_type,
                params: r.params,
                score: r.score,
                question_count: r.question_count,
                duration_seconds: r.duration_seconds,
                created_at: r.created_at,
            })
            .collect();

        Ok(Json(ListQuizResultsResponse { items }))
    }
}

fn default_limit() -> i64 {
    20
}
fn default_offset() -> i64 {
    0
}
