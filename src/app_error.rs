// // src/app_error.rs
// use std::fmt;
// use poem::{http::StatusCode, IntoResponse, Response};
// use poem::Error as PoemError;
// use poem_openapi::ApiResponse;
// use thiserror::Error;
// use sqlx;

// /// Top-level application error used across handlers.
// #[derive(Debug, Error, ApiResponse)]
// pub enum AppError {
//     #[error("database error")]
//     Sqlx(#[from] sqlx::Error),

//     #[error("unauthorized")]
//     Unauthorized,

//     #[error("not found")]
//     NotFound,

//     #[error("internal error: {0}")]
//     Other(String),
// }

// pub type AppResult<T> = Result<T, AppError>;

// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         // Decide status code and body
//         let (status, body_text) = match &self {
//             AppError::Sqlx(e) => {
//                 // Log internal SQL error somewhere suitable
//                 tracing::error!("sqlx error: {:?}", e);
//                 (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
//             }
//             AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
//             AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
//             AppError::Other(msg) => {
//                 tracing::error!("internal error: {}", msg);
//                 (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".to_string())
//             }
//         };

//         // Build a plain text response. You can make JSON if you'd rather.
//         // `poem::Response::builder()` is a straightforward way to set status + body.
//         Response::builder()
//             .status(status)
//             .header("content-type", "text/plain; charset=utf-8")
//             .body(body_text)
//     }
// }

// /// poem-openapi requires the return error type to implement `ApiResponse`.
// /// Delegate to the IntoResponse implementation above.
// impl ApiResponse for AppError {
//     fn meta() -> MetaResponses {
//         todo!()
//     }

//     fn register(registry: &mut Registry) {
//         todo!()
//     }
// }