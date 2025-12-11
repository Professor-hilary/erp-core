use chrono::NaiveDate;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BalanceSheetDto {
    pub section: String,
    pub total_assets: f64, // sqlx::types::BigDecimal or f64 or i64
    pub current_assets: f64,
    pub non_current_assets: f64,
    pub section2: String,
    pub total_liabilities: f64,
    pub current_liabilities: f64,
    pub non_current_liabilities: f64,
    pub section3: String,
    pub total_equity: f64,
}

#[derive(FromRow, Serialize, Debug)]
pub struct IncomeStatementDto {
    pub revenue: sqlx::types::BigDecimal,
    pub cogs: sqlx::types::BigDecimal,
    pub gross_profit: sqlx::types::BigDecimal,
    pub operating_expenses: sqlx::types::BigDecimal,
    pub operating_income: sqlx::types::BigDecimal,
    pub other_income: sqlx::types::BigDecimal,
    pub other_expense: sqlx::types::BigDecimal,
    pub net_income: sqlx::types::BigDecimal,
}

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

#[derive(FromRow, Serialize, Debug)]
pub struct CashFlowDto {
    pub cash_from_operations: sqlx::types::BigDecimal,
    pub add_back_depreciation: sqlx::types::BigDecimal,
    pub decrease_in_ar: sqlx::types::BigDecimal,
    pub increase_in_ap: sqlx::types::BigDecimal,
    pub increase_in_inventory: sqlx::types::BigDecimal,
    pub capex: sqlx::types::BigDecimal,
    pub financing: sqlx::types::BigDecimal,
    pub net_cash_flow: sqlx::types::BigDecimal,
}

#[derive(FromRow, Serialize, Debug)]
pub struct TrialBalanceDto {
    pub account_serial_id: i64,
    pub code: String,
    pub name: String,
    pub r#type: String,
    pub total_debit: sqlx::types::BigDecimal,
    pub total_credit: sqlx::types::BigDecimal,
    pub balance: sqlx::types::BigDecimal,
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
