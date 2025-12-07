use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Vendor {
    pub uuid: Uuid,
    pub name: String,
    pub code: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub credit_limit: bigdecimal::BigDecimal,
    pub current_balance: bigdecimal::BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendor {
    pub name: Option<String>,
    pub code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub credit_limit: Option<bigdecimal::BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBill {
    pub bill_number: String,
    pub vendor_id: i64,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total: bigdecimal::BigDecimal,
    pub items: Option<serde_json::Value>, // JSONB array
}
