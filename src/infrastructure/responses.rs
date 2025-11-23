// src/infrastructure/responses.rs
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub status: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    #[allow(dead_code)]
    BadRequest(String),
    #[allow(dead_code)]
    NotFound, /* (String) */
    Internal(String),
}

impl ApiResponse {
    /// Construct generic json message body for subscribing response codes.
    fn build(
        status: StatusCode,
        message: &str,
        data: Option<Value>,
        meta: Option<Value>,
    ) -> Response {
        (
            status,
            Json(ApiResponse {
                status: status.as_u16(),
                message: message.to_string(),
                data,
                meta,
            }),
        )
            .into_response()
    }

    pub fn success(data: impl serde::Serialize, message: &str) -> Response {
        Self::build(StatusCode::OK, message, Some(json!(data)), None)
    }

    #[allow(unused)]
    pub fn success_with_meta(
        data: impl serde::Serialize,
        message: &str,
        meta: impl serde::Serialize,
    ) -> Response {
        Self::build(
            StatusCode::OK,
            message,
            Some(json!(data)),
            Some(json!(meta)),
        )
    }

    pub fn created(data: impl serde::Serialize, message: &str) -> Response {
        Self::build(StatusCode::CREATED, message, Some(json!(data)), None)
    }

    pub fn created_with_token(
        user: impl serde::Serialize,
        token: String,
        message: &str,
    ) -> Response {
        let payload = json!({
            "user": user,
            "access_token": token,
            "token_type": "Bearer"
        });
        Self::build(StatusCode::CREATED, message, Some(payload), None)
    }

    #[allow(unused)]
    pub fn success_with_token(token: String, message: &str) -> Response {
        let payload = json!({
            "access_token": token,
            "token_type": "Bearer"
        });
        Self::build(StatusCode::OK, message, Some(payload), None)
    }

    #[allow(unused)]
    pub fn empty(status: StatusCode, message: &str) -> Response {
        Self::build(status, message, None, None)
    }

    #[allow(unused)]
    pub fn bad_request(message: &str) -> Response {
        Self::empty(StatusCode::BAD_REQUEST, message)
    }

    #[allow(unused)]
    pub fn unauthorized(message: &str) -> Response {
        Self::empty(StatusCode::UNAUTHORIZED, message)
    }

    #[allow(unused)]
    pub fn forbidden(message: &str) -> Response {
        Self::empty(StatusCode::FORBIDDEN, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized(message) => (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({"status":401, "error":"Unauthorized","message":message,}))
                    .into_response(),
            )
                .into_response(),
            AppError::Forbidden(message) => (
                axum::http::StatusCode::FORBIDDEN,
                Json(json!({"status":403, "error":"Forbidden","message":message,})).into_response(),
            )
                .into_response(),
            AppError::BadRequest(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"status":400, "error":"Bad Request","message":message,}))
                    .into_response(),
            )
                .into_response(),
            AppError::NotFound/* (message) */ => (
                axum::http::StatusCode::NOT_FOUND,
                Json(json!({"status":404, "error":"Not Found","message": "Oops! Resource Not Found",}))
                    .into_response(),
            )
                .into_response(),
            AppError::Internal(message) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status":500, "error":"Internal Server Error","message":message,}))
                    .into_response(),
            )
                .into_response(),

        }
    }
}
