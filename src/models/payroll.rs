// src/models/payrun.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayComponent {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub component_kind: String,
    pub calc_method: String,
    pub default_rate: Option<BigDecimal>,
    pub is_taxable: Option<bool>,
    pub affects_social_security: Option<bool>,
    pub statutory_role: Option<String>,
    pub gl_account_hint: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayComponent {
    pub code: String,
    pub name: String,
    pub component_kind: String,
    pub calc_method: String,
    pub default_rate: Option<BigDecimal>,
    pub is_taxable: Option<bool>,
    pub affects_social_security: Option<bool>,
    pub statutory_role: Option<String>,
    pub gl_account_hint: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payrun {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub status: String,
    pub gl_transaction_uuid: Option<Uuid>,
    pub notes: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrun {
    pub pay_period_start: NaiveDate,
    pub pay_period_end: NaiveDate,
    pub payment_date: NaiveDate,
    pub notes: Option<String>,
    /// If empty / omitted, server generates slips for HR-eligible employees.
    #[serde(default)]
    pub payslips: Vec<CreatePayslip>,
}

#[derive(Debug, Deserialize)]
pub struct PostPayrun {
    pub payrun_serial_id: i64,
    /// Legacy explicit accounts (still used until role resolver is wired everywhere).
    pub labor_expense_id: Option<Uuid>,
    pub income_tax_id: Option<Uuid>,
    pub social_security_id: Option<Uuid>,
    pub cash_account_uuid: Option<Uuid>,
    pub payroll_payable: Option<Uuid>,
    pub overrides: Option<HashMap<String, Uuid>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payslip {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub payrun_uuid: Uuid,
    pub employee_uuid: Uuid,
    pub gross_pay: BigDecimal,
    pub basic_pay: Option<BigDecimal>,
    pub tax_deducted: Option<BigDecimal>,
    pub social_security: Option<BigDecimal>,
    pub employer_social_security: Option<BigDecimal>,
    pub other_deductions: Option<BigDecimal>,
    pub net_pay: Option<BigDecimal>,
    pub payment_method: Option<String>,
    pub bank_account: Option<String>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePayslip {
    pub employee_uuid: Uuid,
    pub gross_pay: BigDecimal,
    pub basic_pay: Option<BigDecimal>,
    pub tax_deducted: Option<BigDecimal>,
    pub social_security: Option<BigDecimal>,
    pub employer_social_security: Option<BigDecimal>,
    pub other_deductions: Option<BigDecimal>,
    pub payment_method: Option<String>,
    pub bank_account: Option<String>,
    #[serde(default)]
    pub benefits: Option<Vec<CreatePayslipItem>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayslipItem {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub payslip_uuid: Uuid,
    pub item_type: String,
    pub description: Option<String>,
    pub component_uuid: Option<Uuid>,
    pub amount: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePayslipItem {
    pub item_type: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub component_uuid: Option<Uuid>,
}

/// Internal row when generating from HR
#[derive(Debug, Clone, FromRow)]
pub struct EligibleEmployeeRow {
    pub employee_uuid: Uuid,
    pub basic_salary: Option<BigDecimal>,
    pub bank_account_number: Option<String>,
    pub payment_method: Option<String>,
}

/*
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
*/
