// src/middleware/auth.rs
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use tower::Layer;
use tower_http::request_id::MakeRequestUuid;
use uuid::Uuid;

use crate::models::dto::JwtClaims;
use crate::routes::AppState;

// Extract user_id from valid JWT → Extension<Authenticated>
#[derive(Clone)]
pub struct Authenticated(pub Uuid);

impl FromRequestParts<Arc<crate::routes::AppState>> for Authenticated {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<crate::routes::AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer ").map(|s| s.to_string()))
            .ok_or(AuthError::MissingToken)?;

        let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_ref());
        let claims = decode::<JwtClaims>(
            &auth_header,
            &decoding_key,
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(Authenticated(claims.claims.sub))
    }
}

// Tower Layer for applying auth to entire routes
pub fn auth_layer(state: Arc<crate::routes::AppState>) -> impl Layer<axum::routing::Route> + Clone + Send + Sync + 'static {
    tower::ServiceBuilder::new()
        .layer(Extension(state))
        .layer(axum::middleware::from_fn(auth_middleware))
}

// Internal middleware function
async fn auth_middleware(
    Extension(user_id): Extension<Authenticated>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, AuthError> {
    let mut req = req;
    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}

// Error responses
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            ),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid JWT token"),
        };
        (status, error_message).into_response()
    }
}