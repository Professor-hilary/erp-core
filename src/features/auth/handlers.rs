// src/features/auth/handlers.rs
use crate::{
    features::auth::{AuthService, repository::PostgresUserRepo},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    models::user::{CreateUser, LoginUser},
    state::AppState,
};
use axum::{Router, extract::State, response::IntoResponse, routing::post};
use serde_json::json;
use std::sync::Arc;
use serde::Deserialize;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
		.route("/logout", post(logout))
        .route("/refresh", post(refresh))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

async fn logout(
    State(state): State<Arc<AppState>>,
    // Prefer extracting the access token from Authorization header
    headers: axum::http::HeaderMap,
    AppJson(payload): AppJson<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let access = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized("Missing access token".into()))?;

    let repo = PostgresUserRepo::new(state.master_pool.clone());
    let service = AuthService::new(repo, state.clone());
    service
        .logout(access, payload.refresh_token.as_deref())
        .await?;

    Ok(ApiResponse::success(json!({}), "Logged out"))
}

/// POST /auth/register
async fn register(
    State(state): State<Arc<AppState>>,
    AppJson(payload): AppJson<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, user_company, (access, refresh)) = service.register(payload, state).await?;

    Ok(ApiResponse::created_with_tokens(
        json!({"user":user, "company":user_company}),
        access, refresh,
        "Registered successfully",
    ))
}

/// POST /auth/login
async fn login(
    State(state): State<Arc<AppState>>,
    AppJson(payload): AppJson<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, user_company, (access, refresh)) = service.login(&payload, state).await?;

    Ok(ApiResponse::success_with_tokens(
        json!({"user": user, "company":user_company}),
        access, refresh,
        "Login successful",
    ))
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// POST /auth/refresh
async fn refresh(
    State(state): State<Arc<AppState>>,
    AppJson(payload): AppJson<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (access, refresh) = service.refresh_tokens(&payload.refresh_token).await?;

    Ok(ApiResponse::success_with_tokens(
        json!({}),
        access, refresh,
        "Token refreshed",
    ))
}
