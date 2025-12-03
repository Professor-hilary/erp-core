// src/models/account.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: Uuid,                  // Required, Auto_gen
    pub serial_id: i64,              // Required, Auto_gen
    pub name: String,                // Required
    pub code: String,                // Required
    pub parent_code: Option<String>, // Optional, Null Default
    pub normal_balance: String,      // Required
    pub is_contra: bool,             // Optional, False Default
    pub is_active: bool,             // Optional, False Default
    pub created_at: DateTime<Utc>,   // Required, Auto_gen
    pub updated_at: DateTime<Utc>,   // Required, Auto_gen
    #[sqlx(rename = "type")] // Required
    pub type_: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CreateAccount {
    pub name: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub code: String,
    pub normal_balance: String,
    pub parent_code: String,
    pub is_contra: bool,
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
