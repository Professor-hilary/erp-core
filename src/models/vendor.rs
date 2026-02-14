use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
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
    pub credit_limit: BigDecimal,
    pub current_balance: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendor {
    pub name: Option<String>,
    pub code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub credit_limit: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Purchase {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub bill_number: String,
    pub vendor_uuid: Uuid,
    pub bill_date: NaiveDate,
    pub due_date: NaiveDate,
    pub reference: Option<String>,
    pub settlement_type: Option<String>,
    pub payment_status: Option<String>,
    pub total_amount: Option<BigDecimal>,
    pub tax_amount: Option<BigDecimal>,
    pub balance_due: Option<BigDecimal>,
    pub posted: bool,
    pub gl_transaction_uuid: Option<Uuid>,
    pub paid_at: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub payment_number: String,
    pub vendor_uuid: Uuid,
    pub payment_date: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub amount: BigDecimal,
    pub applied_amount: BigDecimal,
    pub unapplied_amount: BigDecimal,
    pub gl_transaction_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePurchase {
    pub bill_number: String,
    pub vendor_uuid: Uuid,
    pub bill_date: NaiveDate,
    pub due_date: NaiveDate,
    pub settlement_type: Option<String>,
    pub payment_status: Option<String>,
    pub reference: Option<String>,
    pub tax_amount: Option<BigDecimal>,
    pub total_amount: Option<BigDecimal>,
    pub paid_at: Option<NaiveDate>,
    pub items: Vec<CreatePurchaseItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePayment {
    pub payment_number: String,
    pub vendor_uuid: Uuid,
    pub payment_date: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePurchaseItem {
    pub stock_item_id: i64,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub tax_rate: BigDecimal,
    pub total: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostPurchase {
    pub bill_serial_id: i64,
    pub vat_tax_account: String,
    pub payables_account: Option<String>,
    pub cash_account: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyPayment {
    pub payment_serial_id: i64,
    pub bill_serial_id: i64,
    pub amount: BigDecimal,
}
