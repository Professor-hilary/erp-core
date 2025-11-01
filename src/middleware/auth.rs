// src/middleware/auth.rs
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use crate::routes::AppState;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Clone, Debug)]
pub struct Authenticated(pub Uuid);

// #[async_trait]
impl FromRequestParts<Arc<AppState>> for Authenticated {
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header"))?;

        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid user ID in token"))?;

        Ok(Authenticated(user_id))
    }
}