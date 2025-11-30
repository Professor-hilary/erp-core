// src/features/transactions/handlers.rs
use crate::{
    infrastructure::errors::AppError, features::{
        accounts::repository::PostgresAccountRepo,
        transactions::{repository::PostgresTransactionRepo, services::TransactionService},
    }, infrastructure::responses::ApiResponse, middleware::auth::AuthenticatedTenant, models::transaction::CreateTransaction, state::AppState
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
        .route("/create", post(create_transaction))
        .route("/list", get(get_transactions))
        .route("/{id}", get(get_transaction))
        .route("/{id}", patch(update_transaction))
        .route("/{id}", delete(delete_transaction))
}

async fn create_transaction(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new();
    let tx_repo = PostgresTransactionRepo::new();
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service
        .create_transaction(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(tx, "Transaction created"))
}

async fn get_transactions(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new();
    let tx_repo = PostgresTransactionRepo::new();
    let service = TransactionService::new(acc_repo, tx_repo);
    let txs = service
        .get_transactions(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(txs, "Transactions fetched"))
}

async fn get_transaction(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new();
    let tx_repo = PostgresTransactionRepo::new();
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service
        .get_transaction(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(tx, "Transaction fetched"))
}

async fn update_transaction(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new();
    let tx_repo = PostgresTransactionRepo::new();
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service
        .update_transaction(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(tx, "Transaction updated"))
}

async fn delete_transaction(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let acc_repo: PostgresAccountRepo = PostgresAccountRepo::new();
    let tx_repo: PostgresTransactionRepo = PostgresTransactionRepo::new();
    let service: TransactionService<PostgresAccountRepo, PostgresTransactionRepo> =
        TransactionService::new(acc_repo, tx_repo);
    service
        .delete_transaction(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Transaction deleted"))
}
