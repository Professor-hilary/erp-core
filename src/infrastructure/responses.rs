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

    /// Generic success (200 OK)
    pub fn success(data: impl serde::Serialize, message: &str) -> Response {
        Self::build(StatusCode::OK, message, Some(json!(data)), None)
    }

    /// Created (201) without token (e.g., generic resource creation)
    pub fn created(data: impl serde::Serialize, message: &str) -> Response {
        Self::build(StatusCode::CREATED, message, Some(json!(data)), None)
    }

    /// Created with token in meta (used for registration)
    pub fn created_with_token(
        payload: impl serde::Serialize,
        token: String,
        message: &str,
    ) -> Response {
        let meta = json!({
            "access_token": token,
            "token_type": "Bearer"
        });
        Self::build(
            StatusCode::CREATED,
            message,
            Some(json!(payload)),
            Some(meta),
        )
    }

    /// Success with token in meta (used for login)
    pub fn success_with_token(
        payload: impl serde::Serialize,
        token: String,
        message: &str,
    ) -> Response {
        let meta = json!({
            "access_token": token,
            "token_type": "Bearer"
        });
        Self::build(StatusCode::OK, message, Some(json!(payload)), Some(meta))
    }

    // #[allow(unused)]
    // Optional: Generic with custom meta (pagination, etc.)
    // pub fn success_with_meta(
    //     payload: impl serde::Serialize,
    //     message: &str,
    //     meta: impl serde::Serialize,
    // ) -> Response {
    //     Self::build(
    //         StatusCode::OK,
    //         message,
    //         Some(json!(payload)),
    //         Some(json!(meta)),
    //     )
    // }
}
