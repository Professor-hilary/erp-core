use axum::{
    Router,
    extract::{Json, State},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::*,
    repositories::{PostgresAccountRepo, PostgresTransactionRepo, PostgresUserRepo},
    responses::ApiResponse,
    services::{AccountingService, AuthService},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Claims(String /* , usize */);

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

// --- Auth Handlers ---

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

async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTransaction>,
) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = AccountingService::new(repo, tx_repo);
    let tx = service.create_transaction(user_id, &payload).await?;
    Ok(ApiResponse::created(tx, "Transaction created successfully"))
}

async fn get_transactions(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = AccountingService::new(repo, tx_repo);
    let txs = service.get_transactions(user_id).await?;
    Ok(ApiResponse::success(
        txs,
        "Transactions fetched successfully",
    ))
}

async fn get_trial_balance(State(state): State<Arc<AppState>>) -> Result<Response, AppError> {
    let user_id = Uuid::new_v4(); // TODO: Extract from JWT
    let repo = PostgresAccountRepo::new(state.pool.clone());
    let tx_repo = PostgresTransactionRepo::new(state.pool.clone());
    let service = AccountingService::new(repo, tx_repo);
    let balance = service.get_trial_balance(user_id).await?;
    Ok(ApiResponse::success(
        balance,
        "Trial balance fetched successfully",
    ))
}

// --- Router Setup ---

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "Chiefalry Accountant Running Successfully\n" }),
        )
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .nest(
            "/api/accounts",
            Router::new()
                .route("/create", post(create_account))
                .route("/", get(get_accounts)),
        )
        .nest(
            "/api/transactions",
            Router::new()
                .route("/create", post(create_transaction))
                .route("/", get(get_transactions)),
        )
        .nest(
            "/api/reports",
            Router::new().route("/trial-balance", get(get_trial_balance)),
        )
        .layer(CorsLayer::permissive()) // tighten in production
        .with_state(state)
}
