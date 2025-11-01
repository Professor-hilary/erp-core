use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: u16,
    pub message: String,
    pub data: Option<T>,
}

#[allow(dead_code)] // silence "unused" warnings for now
impl<T: Serialize> ApiResponse<T> {
    /// # Purpose
    /// Generic success response with 200 OK
    pub fn success(data: T, message: &str) -> Response {
        (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: message.to_string(),
                data: Some(data),
            }),
        )
            .into_response()
    }

    /// # Purpose
    /// Resource created response with 201 Created
    pub fn created(data: T, message: &str) -> Response {
        (
            StatusCode::CREATED,
            Json(ApiResponse {
                status: 201,
                message: message.to_string(),
                data: Some(data),
            }),
        )
            .into_response()
    }

    /// Generic OK response, optional alternative for success
    pub fn ok(data: T, message: &str) -> Response {
        (
            StatusCode::OK,
            Json(ApiResponse {
                status: 200,
                message: message.to_string(),
                data: Some(data),
            }),
        )
            .into_response()
    }

    /// Empty response with only status code and message (no data)
    pub fn empty(status: StatusCode, message: &str) -> Response {
        (
            status,
            Json(ApiResponse::<()> {
                status: status.as_u16(),
                message: message.to_string(),
                data: None,
            }),
        )
            .into_response()
    }
}
