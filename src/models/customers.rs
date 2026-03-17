use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Customer {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address: Option<String>,
    pub credit_limit: bigdecimal::BigDecimal,
    pub current_balance: bigdecimal::BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomer {
    pub name: String,
    pub code: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub billing_address: Option<String>,
    pub credit_limit: Option<bigdecimal::BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Turnover {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub invoice_number: String,
    pub settlement_type: String,
    pub customer_uuid: Uuid,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub paid_at:  Option<DateTime<Utc>>,
    pub total_amount: Option<BigDecimal>,
    pub tax_amount: Option<BigDecimal>,
    pub balance_due: Option<BigDecimal>,
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
    pub customer_uuid: Uuid,
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
pub struct CreateTurnover {
    pub invoice_number: String,
    pub customer_uuid: Uuid,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub settlement_type: String,
    pub tax_amount: Option<BigDecimal>,
    pub total_amount: Option<BigDecimal>,
    pub items: Vec<CreateTurnoverItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePayment {
    pub payment_number: String,
    pub customer_uuid: Uuid,
    pub payment_date: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTurnoverItem {
    pub stock_item_id: i64,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub tax_rate: BigDecimal,
    pub total: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostTurnover {
    pub invoice_serial_id: i64,
    pub receivable_code: Option<String>,
    pub revenue_code: String,
    pub cash_account_code: Option<String>,
    pub output_vat_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyPayment {
    pub payment_serial_id: i64,
    pub invoice_serial_id: i64,
    pub amount: BigDecimal,
}
