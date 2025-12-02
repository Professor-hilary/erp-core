// src/routes/transaction.rs

use axum::{
    Extension, Router, extract::{Json, Path, State}, response::IntoResponse, routing::{delete, get, post, put}
};
use uuid::Uuid;
use std::sync::Arc;

use crate::{
    features::transactions::{repository::PostgresTransactionRepo, services::TransactionService}, infrastructure::{errors::AppError, responses::ApiResponse}, middleware::auth::AuthenticatedTenant, models::account::{CreateJournalEntry, UpdateJournalEntry}, state::AppState
};
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_entry))
        .route("/get/{uuid}", get(get_entry))
        .route("/list", get(list_entries))
        .route("/post/{uuid}", put(post_entry))
        .route("/update/{uuid}", put(update_unposted_entry))
        .route("/delete/{uuid}", delete(delete_unposted_entry))
        .route("/void/{uuid}", post(void_entry))
}

async fn create_entry(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateJournalEntry>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    let entry = service.create_journal_entry(&user.tenant_pool, user.user_id, payload).await?;
    Ok(ApiResponse::created(entry, "Journal entry created"))
}

async fn get_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    let entry = service
        .get_journal_entry_with_lines(&user.tenant_pool, uuid, user.user_id)
        .await?
        .ok_or(AppError::NotFound("Entry not found".into()))?;
    Ok(ApiResponse::success(entry, "Journal entry fetched"))
}

async fn list_entries(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    let entries = service.list_journal_entries(&user.tenant_pool, user.user_id, 100, 0).await?;
    Ok(ApiResponse::success(entries, "Journal entries fetched"))
}

async fn post_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    let entry = service.post_journal_entry(&user.tenant_pool, user.user_id, uuid).await?;
    Ok(ApiResponse::success(entry, "Journal entry posted"))
}

async fn update_unposted_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateJournalEntry>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    let entry = service.update_unposted_journal_entry(&user.tenant_pool, user.user_id, uuid, payload).await?;
    Ok(ApiResponse::success(entry, "Unposted journal entry updated"))
}

async fn delete_unposted_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service = TransactionService::new(PostgresTransactionRepo::new());
    service.delete_unposted_journal_entry(&user.tenant_pool, user.user_id, uuid).await?;
    Ok(ApiResponse::success((), "Unposted journal entry deleted"))
}

async fn void_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let reason = body["reason"].as_str().unwrap_or("Voided").to_string();
    let service = TransactionService::new(PostgresTransactionRepo::new());
    service.void_journal_entry(&user.tenant_pool, user.user_id, uuid, reason).await?;
    Ok(ApiResponse::success((), "Journal entry voided with reversing entry"))
}