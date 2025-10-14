// errors.rs
use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use serde::Serialize;

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
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message): (StatusCode, &str, String) = match self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
                e.to_string(),
            ),
            AppError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                "Authentication error",
                msg,
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "Validation error",
                msg,
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Not Found",
                "The requested resource could not be found".to_string(),
            ),
        };

        let error_response = ErrorResponse {
            status: status.as_u16(),
            error: error_type.to_string(),
            message,
        };

        (status, axum::Json(error_response)).into_response()
    }
}
