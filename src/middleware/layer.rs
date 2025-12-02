// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::errors::AppError,
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    models::dto::JwtClaims,
    state::AppState,
};

// Middleware — puts AuthenticatedUser in extensions, fails only on bad token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    // Skip tenant context for company creation
    if path == "/companies" && method == axum::http::Method::POST {
        // Middleware only for create company route
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized(
                "Missing or invalid Authorization header".into(),
            ))?;

        println!("RAW TOKEN RECEIVED: {}", auth_header);

        let token_data: jsonwebtoken::TokenData<JwtClaims> = decode::<JwtClaims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

        // Only insert AuthenticatedUser, ignore tenant claim
        let user_id = token_data.claims.sub;
        request
            .extensions_mut()
            .insert(AuthenticatedUser { user_id });

        return Ok(next.run(request).await);
    } else {
        // All other routes protected by authenticated tenant
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized(
                "Missing or invalid Authorization header".into(),
            ))?;

        println!("RAW TOKEN RECEIVED: {}", auth_header);

        let token_data = decode::<JwtClaims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

        println!(
            "DECODED CLAIMS: company_id:{:?}, tenant_db:{:?}, sub:{:?}",
            token_data.claims.company_id, token_data.claims.tenant_db, token_data.claims.sub
        );

        let user_id = Uuid::parse_str(&token_data.claims.sub.to_string())
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

        // // Always insert the basic user
        request
            .extensions_mut()
            .insert(AuthenticatedUser { user_id });

        // Try to insert tenant context — silently fail (it's optional)
        if let (Some(company_id), Some(tenant_db)) =
            (token_data.claims.company_id, token_data.claims.tenant_db)
        {
            println!("-> Company id and db set successfully");
            if let Some(pool) = state.tenant_pools.get(&company_id) {
                println!("-> Company token set successfully");
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
}
