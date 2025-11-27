// src/middleware/auth.rs

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::responses::AppError,
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    state::AppState,
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    company_id: Option<Uuid>,
    tenant_db: Option<String>,
    exp: usize,
}

// Middleware — puts AuthenticatedUser in extensions, fails only on bad token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized(
            "Missing or invalid Authorization header".into(),
        ))?;

    let token_data = decode::<Claims>(
        auth_header,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    let user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

    // Always insert the basic user
    request
        .extensions_mut()
        .insert(AuthenticatedUser { user_id });

    // Try to insert tenant context — silently fail (it's optional)
    if let (Some(company_id), Some(tenant_db)) =
        (token_data.claims.company_id, token_data.claims.tenant_db)
    {
        if let Some(pool) = state.tenant_pools.get(&company_id) {
            request.extensions_mut().insert(AuthenticatedTenant {
                user_id,
                company_id,
                tenant_db,
                tenant_pool: pool.value().clone(),
            });
        }
    }

    Ok(next.run(request).await)
}
