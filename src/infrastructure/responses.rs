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
}
