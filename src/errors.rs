// errors.rs
use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
struct ErrorResponse {
    status: u16,
    error: String,
    message: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound(String),
    #[error("Not found")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message): (StatusCode, &str, String) = match self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
                e.to_string(),
            ),
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "Authentication error", msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "Validation error", msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "Not Found", msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error", msg),
        };

        let error_response: ErrorResponse = ErrorResponse {
            status: status.as_u16(),
            error: error_type.to_string(),
            message,
        };

        (status, axum::Json(error_response)).into_response()
    }
}
