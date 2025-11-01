// src/features/transactions/handlers.rs
use axum::{
    Extension, Router, extract::{Json, Path, State}, response::Response,
    routing::{get, post, patch, delete},
};
use uuid::Uuid;
use std::sync::Arc;
use crate::{
    errors::AppError,
    features::{
        accounts::repository::PostgresAccountRepo,
        transactions::{repository::PostgresTransactionRepo, services::TransactionService},
    },
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::transaction::CreateTransaction,
    routes::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_transaction))
        .route("/list", get(get_transactions))
        .route("/{id}", get(get_transaction))
        .route("/{id}", patch(update_transaction))
        .route("/{id}", delete(delete_transaction))
}

async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service.create_transaction(user.0, &payload).await?;
    Ok(ApiResponse::created(tx, "Transaction created"))
}

async fn get_transactions(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(acc_repo, tx_repo);
    let txs = service.get_transactions(user.0).await?;
    Ok(ApiResponse::success(txs, "Transactions fetched"))
}

async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service.get_transaction(id, user.0).await?;
    Ok(ApiResponse::success(tx, "Transaction fetched"))
}

async fn update_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(acc_repo, tx_repo);
    let tx = service.update_transaction(id, user.0, &payload).await?;
    Ok(ApiResponse::success(tx, "Transaction updated"))
}

async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let acc_repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(acc_repo, tx_repo);
    service.delete_transaction(id, user.0).await?;
    Ok(ApiResponse::success((), "Transaction deleted"))
}