use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(unused)]
#[derive(sqlx::FromRow, serde::Serialize)]
pub struct AgingEntry {
    pub customer_name: String,
    pub current: i64,
    pub days_30: i64,
    pub days_60: i64,
    pub days_90: i64,
    pub days_120_plus: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BalanceSheetCompareRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub depth: i32,
    pub path: Vec<String>,
    pub balance_1: BigDecimal,
    pub balance_2: BigDecimal,
    pub delta: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BalanceSheetRow {
    pub code: String,
    pub name: String,
    pub category: String,  // asset | liability | equity
    pub depth: i32,        // for indentation
    pub path: Vec<String>, // for ordering + tree logic
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct EquityChangeRow {
    pub code: String,
    pub name: String,
    pub description: String,
    pub amount: f64,
}

#[derive(FromRow, Serialize, Debug)]
pub struct CashbookRowDto {
    pub txn_serial_id: i64,
    pub txn_date: NaiveDate,
    pub reference: Option<String>,
    pub account_uuid: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub debit: sqlx::types::BigDecimal,
    pub credit: sqlx::types::BigDecimal,
    pub memo: Option<String>,
}

#[derive(FromRow, Serialize, Debug)]
pub struct ArAgingDto {
    pub customer_serial_id: i64,
    pub code: String,
    pub name: String,
    pub invoice_serial_id: i64,
    pub invoice_number: String,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: sqlx::types::BigDecimal,
    pub balance_due: sqlx::types::BigDecimal,
    pub days_overdue: i64,
    pub aging_bucket: String,
}

#[derive(FromRow, Serialize, Debug)]
pub struct ApAgingDto {
    pub vendor_serial_id: i64,
    pub code: String,
    pub name: String,
    pub bill_serial_id: i64,
    pub bill_number: String,
    pub bill_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total_amount: sqlx::types::BigDecimal,
    pub balance_due: sqlx::types::BigDecimal,
    pub days_overdue: i64,
    pub aging_bucket: String,
}

#[derive(FromRow, Serialize, Debug)]
pub struct PayrollSummaryDto {
    pub payrun_serial_id: i64,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: Option<NaiveDate>,
    pub employees_paid: i64,
    pub total_gross: sqlx::types::BigDecimal,
    pub total_tax: sqlx::types::BigDecimal,
    pub total_social_security: sqlx::types::BigDecimal,
    pub total_other_deductions: sqlx::types::BigDecimal,
    pub total_net: sqlx::types::BigDecimal,
    pub status: String,
}

#[derive(FromRow, Serialize, Debug)]
pub struct InventoryValuationDto {
    pub item_serial_id: i64,
    pub sku: String,
    pub name: String,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub quantity_on_hand: sqlx::types::BigDecimal,
    pub cost_price: sqlx::types::BigDecimal,
    pub total_value: sqlx::types::BigDecimal,
    pub reorder_level: Option<i64>,
    pub needs_reorder: bool,
}
#[derive(FromRow, Serialize, Debug)]
pub struct CustomerStatementDto {
    pub customer_serial_id: i64,
    pub code: String,
    pub name: String,
    pub invoice_serial_id: Option<i64>,
    pub invoice_number: Option<String>,
    pub issue_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub total_amount: Option<sqlx::types::BigDecimal>,
    pub balance_due: Option<sqlx::types::BigDecimal>,
    pub invoice_status: Option<String>,
    pub payment_serial_id: Option<i64>,
    pub payment_number: Option<String>,
    pub payment_amount: Option<sqlx::types::BigDecimal>,
    pub payment_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrialBalanceRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SimpleReportRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct IncomeStatementRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub depth: i32,
    pub path: Vec<String>,
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FlatAccount {
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub category: String,
    pub normal_balance: String,
    pub is_contra: bool,
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Node {
    pub code: String,
    pub name: String,
    pub category: String,
    pub total: BigDecimal,
    pub children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CashFlowRow {
    pub cash_flow_category: Option<String>,
    pub total: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CashBalanceRow {
    pub opening_cash: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BalanceDeltaRow {
    pub account_type: String,
    pub delta: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NonCashRow {
    pub total: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NetProfitRow {
    pub net_profit: BigDecimal,
}
