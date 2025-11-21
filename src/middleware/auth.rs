// src/middleware/auth.rs
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::responses::AppError;
use crate::state::AppState;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Claims {
    sub: String,
    pub company_id: Option<Uuid>,
    pub tenant_db: Option<String>,
    exp: usize,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Authenticated {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub tenant_db: String,
    pub tenant_pool: PgPool,
}

// #[async_trait]
impl FromRequestParts<Arc<AppState>> for Authenticated {
    // type Rejection = (axum::http::StatusCode, &'static str);
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extract Bearer token
        let auth_header: &str = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized("Missing or invalid Authorization header".into()))?;

        // 2. Decode JWT
        let token_data: jsonwebtoken::TokenData<Claims> = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

        let user_id: Uuid = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

        // 3. Validate company context in token
        let company_id: Uuid = token_data.claims.company_id
            .ok_or(AppError::Forbidden("No company selected. Please use /switch-company".into()))?;

        let tenant_db: String = token_data.claims.tenant_db
            .ok_or(AppError::Forbidden("No tenant database in token".into()))?;

        // 4. Get tenant pool from cache (never create here!)
        let tenant_pool: sqlx::Pool<sqlx::Postgres> = state
            .tenant_pools
            .get(&company_id)
            .ok_or(AppError::Internal("Tenant database not ready".into()))?
            .value()
            .clone();

        Ok(Authenticated {
            user_id,
            company_id,
            tenant_db,
            tenant_pool,
        })
    }
}
    /* // 1. Get token
    // let auth_header = parts
    //     .headers
    //     .get("authorization")
    //     .and_then(|h| h.to_str().ok())
    //     .and_then(|h| h.strip_prefix("Bearer "))
    //     .ok_or(AppError::Unauthorized(
    //         "Missing or invalid Authorization header".into(),
    //     ))?;

    // // 2️⃣ Decode JWT
    // let token_data: jsonwebtoken::TokenData<Claims> = decode::<Claims>(
    //     auth_header,
    //     &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
    //     &Validation::default(),
    // )
    // .map_err(|_| AppError::Unauthorized("Invalid or expired token".into()))?;

    // let user_id = Uuid::parse_str(&token_data.claims.sub)
    //     .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

    // // 3️⃣ Lookup user -> company_id
    // let record = sqlx::query(
    //     r#"
    //     SELECT company_id FROM user_companies WHERE user_id = $1
    //     "#,
    // )
    // .bind(user_id)
    // .fetch_one(&state.master_pool)
    // .await
    // .map_err(|_| AppError::Unauthorized("User has no associated company".into()))?;

    // let company_id: Uuid = record.get("company_id");

    // // 4️⃣ Get company database
    // let company = sqlx::query(
    //     r#"
    //     SELECT db_name FROM companies WHERE id = $1
    //     "#,
    // )
    // .fetch_one(&state.master_pool)
    // .await
    // .map_err(|_| AppError::Unauthorized("User has no associated company".into()))?;

    // let tenant_db: String = company.get("db_name");

    // // 5️⃣ Create tenant pool dynamically
    // let _tenant_db_url = format!(
    //     "postgres://{}:{}@{}:{}/{}",
    //     std::env::var("TENANT_DB_USER").unwrap_or("tenant_user".to_string()),
    //     std::env::var("TENANT_DB_PASSWORD").unwrap_or("tenant_pass".to_string()),
    //     std::env::var("TENANT_DB_HOST").unwrap_or("localhost".to_string()),
    //     std::env::var("TENANT_DB_PORT").unwrap_or("5432".to_string()),
    //     tenant_db
    // );

    // let claims: Claims = token_data.claims;

    // if claims.company_id.is_none() || claims.tenant_db.is_none() {
    //     return Err((
    //         StatusCode::FORBIDDEN,
    //         "No company selected. Please select a company.",
    //     ));
    // }

    // let tenant_pool: sqlx::Pool<sqlx::Postgres> = state
    //     .tenant_pools
    //     .get(&company_id)
    //     .ok_or(AppError::Internal("Tenant database not ready".into()))?
    //     .value()
    //     .clone();

    // // ✅ Return Authenticated
    // Ok(Authenticated {
    //     user_id,
    //     company_id: claims.company_id.unwrap(),
    //     tenant_db: claims.tenant_db.unwrap(),
    //     tenant_pool,
    // })
    */
