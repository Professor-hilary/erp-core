// src/features/auth/handlers.rs
use crate::{
    features::auth::{AuthService, repository::PostgresUserRepo},
    infrastructure::errors::AppError,
    infrastructure::responses::ApiResponse,
    models::user::{CreateUser, LoginUser},
    state::AppState,
};
use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

/// POST /auth/register
async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, user_company, token) = service.register(&payload, state).await?;

    Ok(ApiResponse::created_with_token(
        json!({"user":user, "company":user_company}),
        token,
        "Registered successfully",
        "user",
    ))
}

/// POST /auth/login
async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, user_company, token) = service.login(&payload, state).await?;

    Ok(ApiResponse::success_with_meta(
        json!({"user": user, "company":user_company}),
        "Login successful",
        &token,
    ))
}
