// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::database::tenant_resolver::get_tenant_pool,
    interface::api::errors::AppError,
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    models::dto::{JwtClaims, PeriodRow},
    state::{AppState, PeriodInfo},
};

// Middleware — puts AuthenticatedUser in extensions, fails only on bad token
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path: String = request.uri().path().to_string();
    let method: Method = request.method().clone();

    // =========================================================================================
    // Skip tenant context for company creation
    // =========================================================================================

    // APIs that don't need tenant information
    let is_company_create: bool = method == Method::POST
        && (path.ends_with("/create") && path.contains("/companies") || path == "/companies");

    let is_switch_company: bool = method == Method::POST && path.contains("/switch-company");

    // Middleware only for create company route
    let auth_header: &str = request
        .headers()
        .get("authorization")
        .and_then(|h: &HeaderValue| h.to_str().ok())
        .and_then(|h: &str| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized(
            "You have no valid authorization to perform this action".into(),
        ))?;

    // Check for correct token from user request
    let token_data: TokenData<JwtClaims> = decode::<JwtClaims>(
        auth_header,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid token, kindly login".into()))?;

    // Only insert AuthenticatedUser, ignore tenant claim
    let user_id: Uuid = token_data.claims.sub;
    request
        .extensions_mut()
        .insert(AuthenticatedUser { user_id });

    // =========================================================================================
    // All other routes protected by authenticated tenant
    // =========================================================================================
    if !(is_company_create || is_switch_company)
        && let (Some(company_id), Some(_tenant_db)) =
            (token_data.claims.company_id, token_data.claims.tenant_db)
    {
        let pool = get_tenant_pool(&state, company_id)
            .await
            .map_err(|e: sqlx::Error| AppError::Unauthorized(e.to_string()))?;

        let pool_clone: sqlx::Pool<sqlx::Postgres> = pool.clone(); // cheap Arc clone

        // Make sure period dates are in cache else fetch one
        let period: PeriodInfo = state
            .period_cache
            .try_get_with(company_id, {
                async move {
                    let row_opt: Option<PeriodRow> = sqlx::query_as::<_, PeriodRow>(
                        r#"
                                SELECT uuid, start_date, end_date, is_locked
                                FROM accounting.financial_periods
                                WHERE is_open = true
                                ORDER BY start_date DESC LIMIT 1
                                "#,
                    )
                    .fetch_optional(&pool_clone)
                    .await
                    .map_err(|e: sqlx::Error| anyhow::anyhow!("Database error: {}", e))?;

                    let row: PeriodRow = row_opt.ok_or_else(|| {
                        anyhow::anyhow!("No open financial period for this company")
                    })?;

                    Ok(PeriodInfo {
                        _uuid: row.uuid,
                        start_date: row.start_date,
                        end_date: row.end_date,
                        is_locked: row.is_locked.unwrap_or(false),
                    })
                }
            })
            .await
            .map_err(|cache_err: Arc<anyhow::Error>| {
                AppError::Internal(format!("Period cache failed: {}", cache_err))
            })?;

        request.extensions_mut().insert(AuthenticatedTenant {
            user_id,
            company_id,
            tenant_pool: pool,
            current_period_start: period.start_date,
            current_period_end: period.end_date,
            period_is_locked: period.is_locked,
        });
    }

    Ok(next.run(request).await)
}
