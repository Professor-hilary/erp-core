// src/routes/transactions/handlers.rs

use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::NaiveDate;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::transactions::{repository::PostgresTransactionRepo, services::TransactionService},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::transaction::{
        CreateJournalEntry, JournalEntry, JournalEntryWithLines, LedgerFilter, UpdateJournalEntry,
    },
    state::AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct LedgerQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub posted: Option<bool>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_entry))
        .route("/get/{uuid}", get(get_entry))
        .route("/list/brief", get(list_entries))
        .route("/list", get(list_full_entries))
        .route("/get/ledger/{uuid}", get(get_account_ledger))
        .route("/post/{uuid}", put(post_entry))
        .route("/update/{uuid}", put(update_unposted_entry))
        .route("/delete/{uuid}", delete(delete_unposted_entry))
        .route("/void/{uuid}", post(void_entry))
}

async fn create_entry(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateJournalEntry>,
) -> Result<impl IntoResponse, AppError> {
    tracing::debug!("Post trax api");
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entry: JournalEntryWithLines = service
        .create_journal_entry(&user.tenant_pool.clone(), axum::Extension(user), payload)
        .await?;
    Ok(ApiResponse::created(entry, "Journal entry created"))
}

async fn get_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entry: JournalEntryWithLines = service
        .get_journal_entry_with_lines(&user.tenant_pool, uuid, user.user_id)
        .await?
        .ok_or(AppError::NotFound("Entry not found".into()))?;
    Ok(ApiResponse::success(entry, "Journal entry fetched"))
}

async fn list_entries(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entries: Vec<JournalEntry> = service
        .list_journal_entries(&user.tenant_pool, user.user_id, 100, 0)
        .await?;
    Ok(ApiResponse::success(entries, "Journal entries fetched"))
}

async fn list_full_entries(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entries: Vec<JournalEntryWithLines> = service
        .list_full_journal_entries(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(entries, "Journal entries fetched"))
}

async fn post_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entry: JournalEntryWithLines = service
        .post_journal_entry(&user.tenant_pool, user.user_id, uuid)
        .await?;
    Ok(ApiResponse::success(entry, "Journal entry posted"))
}

async fn update_unposted_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<UpdateJournalEntry>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let entry: JournalEntryWithLines = service
        .update_unposted_journal_entry(&user.tenant_pool, user.user_id, uuid, payload)
        .await?;
    Ok(ApiResponse::success(
        entry,
        "Unposted journal entry updated",
    ))
}

async fn delete_unposted_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<impl IntoResponse, AppError> {
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    service
        .delete_unposted_journal_entry(&user.tenant_pool, user.user_id, uuid)
        .await?;
    Ok(ApiResponse::success((), "Unposted journal entry deleted"))
}

async fn get_account_ledger(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<LedgerQuery>,
) -> Result<impl IntoResponse, AppError> {
    let filter = LedgerFilter {
        account_uuid: uuid,
        from_date: q.from,
        to_date: q.to,
        posted_only: q.posted.unwrap_or(true),
    };

    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    let rows = service.fetch_ledger(&user.tenant_pool, filter).await?;
    Ok(ApiResponse::success(rows, "Fetched ledger for account"))
}

async fn void_entry(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(body): AppJson<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let reason: String = body["reason"].as_str().unwrap_or("Voided").to_string();
    let service: TransactionService<PostgresTransactionRepo> =
        TransactionService::new(PostgresTransactionRepo::new());
    service
        .void_journal_entry(&user.tenant_pool, user.user_id, uuid, reason)
        .await?;
    Ok(ApiResponse::success(
        (),
        "Journal entry voided with reversing entry",
    ))
}
