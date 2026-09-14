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

use crate::{
    features::currency::repository::{CurrencyRepository, PostgresCurrencyRepo},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::currency::{CreateCurrency, UpdateCurrency, UpsertExchangeRate},
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    response::Response,
    routing::{get, patch, post},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/currencies", post(create_currency).get(list_currencies))
        .route("/currencies/{id}", get(get_currency).patch(update_currency))
        .route("/currencies/code/{code}", get(get_currency_by_code))
        .route(
            "/exchange-rates",
            post(upsert_exchange_rate).get(list_exchange_rates),
        )
        .route("/exchange-rates/rate", get(get_rate))
}

async fn create_currency(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateCurrency>,
) -> Result<Response, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = PostgresCurrencyRepo::new();
    let currency = repo
        .create_currency(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(currency, "Currency created"))
}

#[derive(Debug, Deserialize)]
pub struct ListCurrenciesQuery {
    #[serde(default)]
    pub active_only: bool,
}

async fn list_currencies(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<ListCurrenciesQuery>,
) -> Result<Response, AppError> {
    let repo = PostgresCurrencyRepo::new();
    let rows = repo
        .list_currencies(&user.tenant_pool, q.active_only)
        .await?;
    Ok(ApiResponse::success(rows, "Currencies fetched"))
}

async fn get_currency(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = PostgresCurrencyRepo::new();
    let currency = repo.get_currency_by_id(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(currency, "Currency fetched"))
}

async fn get_currency_by_code(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(code): Path<String>,
) -> Result<Response, AppError> {
    let repo = PostgresCurrencyRepo::new();
    let currency = repo
        .get_currency_by_code(&user.tenant_pool, &code)
        .await?;
    Ok(ApiResponse::success(currency, "Currency fetched"))
}

async fn update_currency(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    AppJson(payload): AppJson<UpdateCurrency>,
) -> Result<Response, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = PostgresCurrencyRepo::new();
    let currency = repo
        .update_currency(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::success(currency, "Currency updated"))
}

async fn upsert_exchange_rate(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<UpsertExchangeRate>,
) -> Result<Response, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let repo = PostgresCurrencyRepo::new();
    let rate = repo
        .upsert_exchange_rate(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(rate, "Exchange rate upserted"))
}

#[derive(Debug, Deserialize)]
pub struct ListRatesQuery {
    pub from_currency_id: Option<Uuid>,
    pub to_currency_id: Option<Uuid>,
}

async fn list_exchange_rates(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<ListRatesQuery>,
) -> Result<Response, AppError> {
    let repo = PostgresCurrencyRepo::new();
    let rows = repo
        .list_exchange_rates(
            &user.tenant_pool,
            q.from_currency_id,
            q.to_currency_id,
        )
        .await?;
    Ok(ApiResponse::success(rows, "Exchange rates fetched"))
}

#[derive(Debug, Deserialize)]
pub struct GetRateQuery {
    pub from_currency_id: Uuid,
    pub to_currency_id: Uuid,
    pub on: NaiveDate,
}

async fn get_rate(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<GetRateQuery>,
) -> Result<Response, AppError> {
    let repo = PostgresCurrencyRepo::new();
    let rate = repo
        .get_rate(
            &user.tenant_pool,
            q.from_currency_id,
            q.to_currency_id,
            q.on,
        )
        .await?;
    Ok(ApiResponse::success(rate, "Rate fetched"))
}
