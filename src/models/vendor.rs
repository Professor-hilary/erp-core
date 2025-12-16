use bigdecimal::BigDecimal;
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bill {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub bill_number: String,
    pub vendor_uuid: Uuid,
    pub bill_date: NaiveDate,
    pub due_date: NaiveDate,
    pub reference: Option<String>,
    pub total_amount: BigDecimal,
    pub tax_amount: BigDecimal,
    pub balance_due: BigDecimal,
    pub currency: String,
    pub status: String,
    pub posted: bool,
    pub gl_transaction_uuid: Option<Uuid>,
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
    pub currency: String,
    pub applied_amount: BigDecimal,
    pub unapplied_amount: BigDecimal,
    pub gl_transaction_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBill {
    pub bill_number: String,
    pub vendor_uuid: Uuid,
    pub bill_date: NaiveDate,
    pub due_date: NaiveDate,
    pub reference: Option<String>,
    pub currency: String,
    pub tax_amount: BigDecimal,
    pub total_amount: BigDecimal,
    pub items: Vec<CreateBillItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePayment {
    pub payment_number: String,
    pub vendor_uuid: Uuid,
    pub payment_date: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub amount: BigDecimal,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBillItem {
    pub item_code: Option<String>,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub tax_rate: BigDecimal,
    pub total: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostBill {
    pub bill_serial_id: i64,
    pub inventory_account: String,
    pub payables_account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyPayment {
    pub payment_serial_id: i64,
    pub bill_serial_id: i64,
    pub amount: BigDecimal,
}
