use axum::{
    Router,
    extract::{Json, State},
    response::Response,
    routing::post,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    features::{
        accounts::{repository::PostgresAccountRepo, service::AccountingService},
        transactions::repository::PostgresTransactionRepo,
    },
    infrastructure::responses::ApiResponse,
    models::account::CreateAccount,
    routes::AppState,
};
use jsonwebtoken::{DecodingKey, Validation, decode};

#[derive(Debug, Deserialize, Clone)]
struct Claims(String /* , usize */);

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create_account", post(create_account))
        .route("/get_accounts", post(get_accounts))
}

// Helper to extract user_id from JWT token
fn extract_user_id(token: &str, secret: &str) -> Result<Uuid, AppError> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Auth("Invalid token".into()))?;

    Uuid::parse_str(&decoded.claims.0)
        .map_err(|_| AppError::Auth("Invalid user ID in token".into()))
}

// --- Accounting Handlers ---
async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAccount>,
) -> Result<Response, AppError> {
    // TODO: Extract token from header properly
    let token = "dummy_token";
    let user_id = extract_user_id(token, &state.jwt_secret)?;

    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = AccountingService::new(repo, tx_repo);
    let account = service.create_account(user_id, &payload).await?;
    Ok(ApiResponse::created(
        account,
        "Account created successfully",
    ))
}

async fn get_accounts(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = AccountingService::new(repo, tx_repo);
    let accounts = service.get_accounts(user_id).await?;
    Ok(ApiResponse::success(
        accounts,
        "Accounts fetched successfully",
    ))
}
