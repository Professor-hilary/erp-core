use axum::{
    Router, extract::{Json, State}, response::Response, routing::post
};
use std::sync::Arc;

use crate::{
    errors::AppError,
    features::auth::{AuthService, repository::PostgresUserRepo},
    infrastructure::responses::ApiResponse,
    models::user::{CreateUser, LoginUser},
    routes::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Response, AppError> {
    let repo = PostgresUserRepo::new(state.pool.clone());
    let service = AuthService::new(repo, state.jwt_secret.clone());
    let user = service.register(&payload).await?;
    Ok(ApiResponse::created(user, "User registered successfully"))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUser>,
) -> Result<Response, AppError> {
    let repo = PostgresUserRepo::new(state.pool.clone());
    let service = AuthService::new(repo, state.jwt_secret.clone());
    let token = service.login(&payload).await?;
    Ok(ApiResponse::success(token, "Login successful"))
}
