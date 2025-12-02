// src/middleware/debug_state.rs
use crate::{middleware::auth::AuthenticatedUser, models::dto::JwtClaims, state::AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::info;

pub async fn debug_app_state_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    // 1. Decode the JWT **without verifying signature** just to see the claims (dev only!)
    let jwt_company_id = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| {
            // This is the new 2024–2025 way: use decode with empty validation
            use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
            let mut validation = Validation::new(Algorithm::HS256);
            #[allow(deprecated)]
            validation.insecure_disable_signature_validation(); // removes signature check
            validation.required_spec_claims.remove("exp"); // ignore expiry too
            decode::<JwtClaims>(token, &DecodingKey::from_secret(&[]), &validation)
                .ok()
                .map(|data| {
                    data.claims
                        .company_id
                        .map(|id| id.to_string())
                        .unwrap_or("none".into())
                })
        })
        .unwrap_or("cannot_decode".into());

    // 2. Show all cached tenant pools + their health
    let cached: Vec<_> = state
        .tenant_pools
        .iter()
        .map(|entry| {
            let company_id = *entry.key();
            let pool = entry.value();
            // sqlx 0.7+ → use .size() and .num_idle() instead of .status()
            let size = pool.size();
            let idle = pool.num_idle();
            (company_id, size, idle)
        })
        .collect();

    info!(
        method = %request.method(),
        uri = %request.uri(),
        user_id = request
            .extensions()
            .get::<AuthenticatedUser>()
            .map(|u| u.user_id.to_string())
            .unwrap_or("none".into()),
        jwt_claimed_company = jwt_company_id,
        cached_tenant_count = state.tenant_pools.len(),
        cached_pools = ?cached,
        tenant_config_base_url = %state.tenant_config.base_url,
        "AppState snapshot (debug)"
    );

    next.run(request).await
}
