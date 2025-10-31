// src/feayures/transactions/handlers.rs
use axum::{
    Router,
    extract::{Json, State},
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    features::{
        accounts::repository::PostgresAccountRepo,
        transactions::{repository::PostgresTransactionRepo, services::TransactionService},
    },
    infrastructure::responses::ApiResponse,
    models::transaction::CreateTransaction,
};

use crate::routes::AppState;

//#[derive(Debug, Deserialize, Clone)]
//struct Claims(String /* , usize */);

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_transaction))
        .route("/list", get(get_transactions))
}

// --- Auth Handlers ---
async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(repo, tx_repo);
    let tx = service.create_transaction(user_id, &payload).await?;
    Ok(ApiResponse::created(tx, "Transaction created successfully"))
}

async fn get_transactions(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = TransactionService::new(repo, tx_repo);
    let txs = service.get_transactions(user_id).await?;
    Ok(ApiResponse::success(
        txs,
        "Transactions fetched successfully",
    ))
}
