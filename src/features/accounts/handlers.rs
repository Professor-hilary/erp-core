// src/features/accounts/handlers.rs
use crate::{
    features::accounts::{repository::PostgresAccountRepo, service::AccountingService},
    infrastructure::errors::AppError,
    infrastructure::responses::ApiResponse,
    middleware::auth::AuthenticatedTenant,
    models::account::CreateAccount,
    state::AppState,
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
        .route("/get/uuid/{id}", get(get_account_by_uuid))
        .route("/get/serial/{id}", get(get_account_by_serial))
        .route("/update/{id}", patch(update_account))
        .route("/delete/{id}", delete(delete_account))
}

async fn create_account(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateAccount>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    let account = service
        .create_account(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(account, "Account created"))
}

async fn get_accounts(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    let accounts = service
        .get_accounts(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(accounts, "Accounts fetched"))
}

async fn get_account_by_uuid(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    let account = service
        .get_account_by_uuid(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn get_account_by_serial(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    let account = service
        .get_account_by_serial(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn update_account(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateAccount>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    let account = service
        .update_account(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(account, "Account updated"))
}

async fn delete_account(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresAccountRepo::new();
    let service = AccountingService::new(repo);
    service
        .delete_account(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Account deleted"))
}
