// src/middleware/auth.rs
use axum::{
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    infrastructure::{database::tenant_resolver::get_tenant_pool, errors::AppError},
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    models::dto::JwtClaims,
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
            .and_then(|h: &axum::http::HeaderValue| h.to_str().ok())
            .and_then(|h: &str| h.strip_prefix("Bearer "))
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
        // Inside your auth_middleware else branch, after getting pool

        match (token_data.claims.company_id, token_data.claims.tenant_db) {
            (Some(company_id), Some(_tenant_db)) => {
                let pool = get_tenant_pool(&state, company_id)
                    .await
                    .map_err(|e| AppError::Unauthorized(e.to_string()))?;

                let pool_clone = pool.clone(); // cheap Arc clone

                let period = state
                    .period_cache
                    .try_get_with(company_id, {
                        async move {
                            #[derive(sqlx::FromRow)]
                            struct PeriodRow {
                                uuid: Uuid,
                                start_date: chrono::NaiveDate,
                                end_date: chrono::NaiveDate,
                                is_locked: Option<bool>,
                            }

                            let row_opt: Option<PeriodRow> = sqlx::query_as::<_, PeriodRow>(
                                r#"
                                SELECT uuid, start_date, end_date, is_locked
                                FROM accounting.financial_periods
                                WHERE company_id = $1
                                AND is_open = true
                                ORDER BY start_date DESC
                                LIMIT 1
                                "#,
                            )
                            .bind(company_id)
                            .fetch_optional(&pool_clone)
                            .await
                            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?; // ← use anyhow or Box<dyn Error>

                            let row = row_opt.ok_or_else(|| {
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
            _ => (),
        }

        Ok(next.run(request).await)
    }
}
