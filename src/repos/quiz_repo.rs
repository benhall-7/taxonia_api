use anyhow::Result;
use serde_json::Value;
use sqlx::{PgPool, FromRow};

#[derive(Clone)]
pub struct QuizRepo {
    pool: PgPool,
}

impl QuizRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_quiz_result(
        &self,
        user_id: i64,
        quiz_type: &str,
        params: &Value,
        score: f64,
        question_count: Option<i32>,
        duration_seconds: Option<i32>,
    ) -> Result<i64> {
        let rec: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO quiz_results (
                user_id, quiz_type, params, score, question_count, duration_seconds
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(quiz_type)
        .bind(params)
        .bind(score)
        .bind(question_count)
        .bind(duration_seconds)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.0)
    }

    pub async fn list_quiz_results_for_user(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QuizResultRow>> {
        let rows = sqlx::query_as::<_, QuizResultRow>(
            r#"
            SELECT
                id,
                quiz_type,
                params,
                score,
                question_count,
                duration_seconds,
                created_at
            FROM quiz_results
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

#[derive(FromRow)]
pub struct QuizResultRow {
    pub id: i64,
    pub quiz_type: String,
    pub params: serde_json::Value,
    pub score: f64,
    pub question_count: Option<i32>,
    pub duration_seconds: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
