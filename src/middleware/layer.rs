// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use sqlx::{Error, Postgres};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::{database::tenant_resolver::get_tenant_pool, errors::AppError},
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
    let path: String = request.uri().path().to_string();
    let method: Method = request.method().clone();

    let is_company_create: bool = method == Method::POST
        && (path.ends_with("/create") && path.contains("/companies") || path == "/companies");

    let is_switch_company: bool = method == Method::POST && path.contains("/switch-company");

    // Skip tenant context for company creation
    if is_company_create || is_switch_company {
        // Middleware only for create company route
        let auth_header = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized(
                "You need to log in to perform this action".into(),
            ))?;

        println!("RAW TOKEN RECEIVED: {}", auth_header);

        let token_data: TokenData<JwtClaims> = decode::<JwtClaims>(
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
        let auth_header: &str = request
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized(
                "You have no valid authorization to perform this action!".into(),
            ))?;

        let token_data: TokenData<JwtClaims> = decode::<JwtClaims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

        println!(
            "DECODED CLAIMS: company_id:{:?}, tenant_db:{:?}, sub:{:?}",
            token_data.claims.company_id, token_data.claims.tenant_db, token_data.claims.sub
        );

        let user_id: Uuid = Uuid::parse_str(&token_data.claims.sub.to_string())
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

        // Always insert the basic user
        request
            .extensions_mut()
            .insert(AuthenticatedUser { user_id });

        // Try to insert tenant context — silently fail (it's optional)
        match (token_data.claims.company_id, token_data.claims.tenant_db) {
            (Some(company_id), Some(_tenant_db)) => {
                let pool: sqlx::Pool<Postgres> = get_tenant_pool(&state, company_id)
                    .await
                    .map_err(|e: Error| AppError::Unauthorized(e.to_string()))?;

                request.extensions_mut().insert(AuthenticatedTenant {
                    user_id,
                    company_id,
                    tenant_pool: pool,
                });
            }
            _ => (),
        }
        Ok(next.run(request).await)
    }
}
