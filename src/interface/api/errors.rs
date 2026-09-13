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
                Json(json!({
                    "status": 401,
                    "error": "Unauthorized",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "status": 403,
                    "error": "Forbidden",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": 400,
                    "error": "Bad Request",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "status": 404,
                    "error": "Not Found",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": 500,
                    "error": "Internal Server Error",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::Unprocessable(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "status": 422,
                    "error": "Bad data",
                    "message": message,
                })),
            )
                .into_response(),

            AppError::Database(error) => {
                let (status, message) = match &error {
                    // -------------------------------------------------------
                    // Postgres-raised exceptions (your RAISE EXCEPTION …)
                    // -------------------------------------------------------
                    sqlx::Error::Database(db_err) => {
                        let msg = db_err.message().to_string();
                        let status = match db_err.code().as_deref() {
                            // Custom codes you raise from post_transaction etc.
                            Some("P0001") | Some("P0002") | Some("P0003") | Some("P0004") => {
                                StatusCode::BAD_REQUEST
                            }
                            // Common Postgres constraint / data errors
                            Some("23505") => StatusCode::CONFLICT,          // unique_violation
                            Some("23503") => StatusCode::BAD_REQUEST,       // foreign_key_violation
                            Some("23502") => StatusCode::BAD_REQUEST,       // not_null_violation
                            Some("22P02") => StatusCode::BAD_REQUEST,       // invalid_text_representation
                            Some("22001") => StatusCode::BAD_REQUEST,       // string_data_right_truncation
                            Some("22003") => StatusCode::BAD_REQUEST,       // numeric_value_out_of_range
                            Some("P0001") => StatusCode::BAD_REQUEST,       // raise_exception (generic)
                            _ => StatusCode::INTERNAL_SERVER_ERROR,
                        };
                        (status, msg)
                    }

                    // -------------------------------------------------------
                    // Connection / pool / driver level
                    // -------------------------------------------------------
                    sqlx::Error::Configuration(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database configuration error: {e}"),
                    ),
                    sqlx::Error::Io(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database I/O error: {e}"),
                    ),
                    sqlx::Error::Tls(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database TLS error: {e}"),
                    ),
                    sqlx::Error::Protocol(msg) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database protocol error: {msg}"),
                    ),
                    sqlx::Error::PoolTimedOut => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Database connection pool timed out".into(),
                    ),
                    sqlx::Error::PoolClosed => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Database connection pool is closed".into(),
                    ),
                    sqlx::Error::WorkerCrashed => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database worker crashed".into(),
                    ),
                    sqlx::Error::BeginFailed => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to begin database transaction".into(),
                    ),

                    // -------------------------------------------------------
                    // Query / result shape problems (usually programming bugs)
                    // -------------------------------------------------------
                    sqlx::Error::RowNotFound => (
                        StatusCode::NOT_FOUND,
                        "Requested database row was not found".into(),
                    ),
                    sqlx::Error::TypeNotFound { type_name } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database type not found: {type_name}"),
                    ),
                    sqlx::Error::ColumnIndexOutOfBounds { index, len } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Column index {index} out of bounds (len = {len})"),
                    ),
                    sqlx::Error::ColumnNotFound(name) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Column not found: {name}"),
                    ),
                    sqlx::Error::ColumnDecode { index, source } => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to decode column {index}: {source}"),
                    ),
                    sqlx::Error::Encode(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to encode value for database: {e}"),
                    ),
                    sqlx::Error::Decode(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to decode value from database: {e}"),
                    ),
                    sqlx::Error::AnyDriverError(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database driver error: {e}"),
                    ),

                    // -------------------------------------------------------
                    // Migration & save-point (rare in request path)
                    // -------------------------------------------------------
                    sqlx::Error::Migrate(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database migration error: {e}"),
                    ),
                    sqlx::Error::InvalidSavePointStatement => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid save-point statement".into(),
                    ),

                    // -------------------------------------------------------
                    // Catch-all for any future variants
                    // -------------------------------------------------------
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unexpected database error".into(),
                    ),
                };

                (
                    status,
                    Json(json!({
                        "status": status.as_u16(),
                        "error": "Database error",
                        "message": capitalize(&message),
                    })),
                )
                    .into_response()
            }
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
                           }
