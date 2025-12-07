// models/transaction.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

#[allow(unused)]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub txn_date: DateTime<Utc>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub posted: bool,
    pub module: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TransactionEntry {
    pub uuid: Uuid,
    pub transaction_uuid: Uuid,
    pub account_uuid: Uuid,
    pub line_no: i32,
    pub amount: BigDecimal,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub memo: Option<String>,
}
