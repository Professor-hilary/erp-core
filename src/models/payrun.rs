// src/models/payrun.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Payrun {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub status: String,
    pub gl_transaction_uuid: Option<Uuid>,
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

#[derive(Debug, Deserialize)]
pub struct PostPayrun {
    pub payrun_serial_id: i64,    // ID for pending payrun
    pub labor_expense_id: Uuid,   // Direct or Indirect labor expense
    pub income_tax_id: Uuid,      // Income Tax Liability
    pub social_security_id: Uuid, // Social Security Tax Liability
    pub cash_account_uuid: Uuid,  // Cash/Bank Account
    pub payroll_payable: Uuid,    // Payroll Liability Unpaid
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Payslip {
    pub uuid: Uuid,
    pub payrun_uuid: Uuid,
    pub employee_uuid: Uuid,
    pub gross_pay: bigdecimal::BigDecimal,
    pub tax_deducted: bigdecimal::BigDecimal,
    pub social_security: bigdecimal::BigDecimal,
    pub other_deductions: bigdecimal::BigDecimal,
    pub net_pay: bigdecimal::BigDecimal,
    pub payment_method: String,
    pub bank_account: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayslip {
    pub employee_uuid: Uuid,
    pub gross_pay: bigdecimal::BigDecimal,
    pub tax_deducted: Option<bigdecimal::BigDecimal>,
    pub social_security: Option<bigdecimal::BigDecimal>,
    pub other_deductions: Option<bigdecimal::BigDecimal>,
    pub payment_method: Option<String>,
    pub bank_account: Option<String>,
    pub benefits: Option<Vec<PayslipItem>>,
}

#[derive(Debug, Deserialize)]
pub struct PayslipItem {
    pub item_type: String,
    pub description: Option<String>,
    pub amount: bigdecimal::BigDecimal,
}
