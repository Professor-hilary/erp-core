use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::BigDecimal};
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub balance: BigDecimal,  // ← Changed from f64 (or rust_decimal::Decimal)
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub type_: String,
}