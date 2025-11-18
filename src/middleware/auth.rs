// src/middleware/auth.rs
use crate::routes::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::Deserialize;
use sqlx::Row;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct Claims {
    sub: String,
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
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 1. Get token
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or((
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            ))?;

        // 2️⃣ Decode JWT
        let token_data = decode::<Claims>(
            auth_header,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
            )
        })?;

        let user_id = Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid user ID in token",
            )
        })?;

        // 3️⃣ Lookup user -> company_id
        let record = sqlx::query(
            r#"
            SELECT company_id FROM user_companies WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&state.master_pool)
        .await
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "User has no company"))?;

        let company_id: Uuid = record.get("company_id");

        // 4️⃣ Get company database
        let company = sqlx::query(
            r#"
            SELECT db_name FROM companies WHERE id = $1
            "#,
        )
        .fetch_one(&state.master_pool)
        .await
        .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "User has no company"))?;

        let tenant_db: String = company.get("db_name");

        // 5️⃣ Create tenant pool dynamically
        let tenant_db_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            std::env::var("TENANT_DB_USER").unwrap_or("tenant_user".to_string()),
            std::env::var("TENANT_DB_PASSWORD").unwrap_or("tenant_pass".to_string()),
            std::env::var("TENANT_DB_HOST").unwrap_or("localhost".to_string()),
            std::env::var("TENANT_DB_PORT").unwrap_or("5432".to_string()),
            tenant_db
        );

        let tenant_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&tenant_db_url)
            .await
            .map_err(|_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to connect to tenant DB",
                )
            })?;

        // ✅ Return Authenticated
        Ok(Authenticated {
            user_id,
            company_id,
            tenant_db,
            tenant_pool,
        })
    }
}
