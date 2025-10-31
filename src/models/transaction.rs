// models.rs
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub description: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: f64,
    pub date: NaiveDate,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransaction {
    pub description: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: f64,
}