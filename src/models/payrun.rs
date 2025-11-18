// src/models/payrun.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Payrun {
    pub payrun_id: i32,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub status: String,
    pub gl_transaction_id: Option<i32>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrun {
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub notes: Option<String>,
    pub payslips: Vec<CreatePayslip>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Payslip {
    pub payslip_id: i32,
    pub payrun_id: i32,
    pub employee_id: i32,
    pub gross_pay: bigdecimal::BigDecimal,
    pub tax_deducted: bigdecimal::BigDecimal,
    pub nssf: bigdecimal::BigDecimal,
    pub other_deductions: bigdecimal::BigDecimal,
    pub net_pay: bigdecimal::BigDecimal,
    pub payment_method: String,
    pub bank_account: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayslip {
    pub employee_id: i32,
    pub gross_pay: bigdecimal::BigDecimal,
    pub tax_deducted: Option<bigdecimal::BigDecimal>,
    pub nssf: Option<bigdecimal::BigDecimal>,
    pub other_deductions: Option<bigdecimal::BigDecimal>,
    pub payment_method: Option<String>,
    pub bank_account: Option<String>,
    pub items: Option<Vec<PayslipItem>>,
}

#[derive(Debug, Deserialize)]
pub struct PayslipItem {
    pub item_type: String,
    pub description: Option<String>,
    pub amount: bigdecimal::BigDecimal,
}
