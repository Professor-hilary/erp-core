// errors.rs
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::Error;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] Error),
    #[error("Auth error: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    BadRequest(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Server error: {0}")]
    Internal(String),
    #[error("Not authorized: {0}")]
    Forbidden(String),
    #[error("Bad data: {0}")]
    Unprocessable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"status":401, "error":"Unauthorized","message":message,})),
            )
                .into_response(),
            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(json!({"status":403, "error":"Forbidden","message":message,})),
            )
                .into_response(),
            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"status":400, "error":"Bad Request","message":message,})),
            )
                .into_response(),
            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(json!({"status":404, "error":"Not Found","message": message,})),
            )
                .into_response(),
            AppError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status":500, "error":"Internal Server Error","message":message,})),
            )
                .into_response(),
            AppError::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status":500,
                    "error":"Database error",
                    "message":capitalize(match &error {
                        sqlx::Error::Database(db_err) => db_err.message(),
                        _ => "Unexpected database error",
                    },
                )})),
            )
                .into_response(),
            AppError::Unprocessable(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"status":422, "error":"Bad data","message":message,})),
            )
                .into_response(),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars: std::str::Chars<'_> = s.chars();

    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
