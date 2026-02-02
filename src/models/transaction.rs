// models/transaction.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionLine {
    pub account_uuid: Uuid, // or keep code/serial_id and resolve in service
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransaction {
    pub txn_date: DateTime<Utc>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub module: Option<String>, // e.g., "journal", "invoice", "payment"
    pub lines: Vec<CreateTransactionLine>, // MUST have >= 2 lines
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLineInput {
    pub account_uuid: Uuid,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJournalEntry {
    pub txn_date: NaiveDate,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub module: Option<String>, // "journal", "invoice", "payment", etc.
    pub posted: bool,
    pub lines: Vec<TransactionLineInput>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct JournalEntry {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub txn_date: NaiveDate,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub posted: bool,
    pub module: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct JournalEntryLine {
    pub uuid: Uuid,
    pub transaction_uuid: Uuid,
    pub account_uuid: Uuid,
    pub line_no: i32,
    pub amount: BigDecimal,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub memo: Option<String>,
    // Fields from accounts table
    pub account_name: String,
    pub account_code: String,
    pub account_category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JournalEntryWithLines {
    pub header: JournalEntry,
    pub lines: Vec<JournalEntryLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJournalEntry {
    pub txn_date: NaiveDate,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub module: Option<String>,
    pub lines: Option<Vec<TransactionLineInput>>, // Optional: only update if provided
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct LedgerRowDto {
    pub transaction_uuid: Uuid,
    pub transaction_serial_id: i64,
    pub entry_uuid: Uuid,
    pub entry_serial_id: i64,
    pub txn_date: NaiveDate,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub amount: BigDecimal,
    pub memo: Option<String>,
    pub created_at: DateTime<Utc>,
    pub running_balance: BigDecimal,
}

pub struct LedgerFilter {
    pub account_uuid: Uuid,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub posted_only: bool,
}
