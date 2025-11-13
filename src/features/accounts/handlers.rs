// src/features/accounts/handlers.rs
use crate::{
    errors::AppError,
    features::accounts::{repository::PostgresAccountRepo, service::AccountingService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::account::CreateAccount,
    routes::AppState,
};
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_account))
        .route("/list", get(get_accounts))
        .route("/uuid/{id}", get(get_account_by_uuid))
        .route("/{id}", get(get_account_by_serial))
        .route("/{id}", patch(update_account))
        .route("/{id}", delete(delete_account))
}

async fn create_account(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateAccount>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    let account = service.create_account(user.0, &payload).await?;
    Ok(ApiResponse::created(account, "Account created"))
}

async fn get_accounts(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    let accounts = service.get_accounts(user.0).await?;
    Ok(ApiResponse::success(accounts, "Accounts fetched"))
}

async fn get_account_by_uuid(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    let account = service.get_account_by_uuid(id, user.0).await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn get_account_by_serial(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    let account = service.get_account_by_serial(id, user.0).await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn update_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateAccount>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    let account = service.update_account(id, user.0, &payload).await?;
    Ok(ApiResponse::success(account, "Account updated"))
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let service = AccountingService::new(repo);
    service.delete_account(id, user.0).await?;
    Ok(ApiResponse::success((), "Account deleted"))
}
