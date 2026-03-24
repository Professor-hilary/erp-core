// src/features/accounts/handlers.rs
use crate::{
    features::accounts::{repository::PostgresAccountRepo, services::AccountingService},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::{
        account::{Account, CreateAccount},
        dto::FinancialPeriodDto,
    },
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Path, State},
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
        .route("/period/close/{id}", patch(close_period))
        .route("/period/list", get(get_financial_periods))
}

async fn create_account(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAccount>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let account: Account = service
        .create_account(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(account, "Account created"))
}

async fn get_accounts(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let accounts: Vec<Account> = service
        .get_accounts(&user.tenant_pool, user.user_id, state)
        .await?;
    Ok(ApiResponse::success(accounts, "Accounts fetched"))
}

async fn get_financial_periods(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let accounts: Vec<FinancialPeriodDto> = service.list_all_periods(&user.tenant_pool).await?;
    Ok(ApiResponse::success(accounts, "Accounts fetched"))
}

async fn get_account_by_uuid(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let account: Account = service
        .get_account_by_uuid(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn get_account_by_serial(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let account: Account = service
        .get_account_by_serial(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(account, "Account fetched"))
}

async fn update_account(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAccount>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    let account: Account = service
        .update_account(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(account, "Account updated"))
}

async fn delete_account(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    service
        .delete_account(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Account deleted"))
}

async fn close_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let service: AccountingService<PostgresAccountRepo> = AccountingService::new(repo);
    service
        .close_financial_period(state, &user.tenant_pool, period_id, user.company_id)
        .await?;
    Ok(ApiResponse::success(
        (),
        "Financial period closed successfully!",
    ))
}
