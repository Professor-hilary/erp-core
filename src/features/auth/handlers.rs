// src/features/auth/handlers.rs
use crate::{
    features::auth::{AuthService, repository::PostgresUserRepo},
    infrastructure::errors::AppError,
    infrastructure::responses::ApiResponse,
    middleware::auth::AuthenticatedUser,
    models::user::{CreateUser, LoginUser},
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::post,
};
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/switch-company", post(switch_company))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, token) = service.register(&payload, state).await?;

    Ok(ApiResponse::created_with_token(
        user,
        token,
        "Registered successfully",
        "user",
    ))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let (user, token) = service.login(&payload, state).await?;

    Ok(ApiResponse::success_with_meta(
        user,
        "Login successful",
        &token,
    ))
}

// POST /api/auth/switch-company { "company_id": "..." }
async fn switch_company(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let company_id: Uuid = payload["company_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::BadRequest("Invalid company_id".into()))?;

    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let new_token: String = service
        .switch_company(user.user_id, company_id, state)
        .await?;

    Ok(ApiResponse::success(
        new_token,
        "Company switched successfully",
    ))
}
