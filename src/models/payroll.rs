// src/models/payrun.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// PAYROLL SNAPSHOT (read model for generate payslips — optional FromRow)
// =============================================================================

/*#[derive(Debug, Clone, FromRow, Serialize)]
pub struct EmployeePayrollSnapshot {
    pub employee_uuid: Uuid,
    pub status: String,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub last_working_date: Option<NaiveDate>,
    pub basic_salary: Option<BigDecimal>,
    pub pay_frequency: Option<String>,
    pub employment_type: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub payment_method: Option<String>,
    pub currency_code: Option<String>,
    pub tax_id: Option<String>,
    pub social_security_number: Option<String>,
}*/

// =============================================================================
// PAY COMPONENTS
// =============================================================================

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

// =============================================================================
// PAYRUNS / PAYSLIPS
// =============================================================================

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
    pub posting_snapshot: Option<serde_json::Value>,
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
    /// If empty, service may generate slips from HR eligibility snapshot.
    #[serde(default)]
    pub payslips: Vec<CreatePayslip>,
}

#[derive(Debug, Deserialize)]
pub struct PostPayrun {
    pub payrun_serial_id: i64,
    /// Legacy explicit accounts (optional once role mapping is live).
    pub labor_expense_id: Option<Uuid>,
    pub income_tax_id: Option<Uuid>,
    pub social_security_id: Option<Uuid>,
    pub cash_account_uuid: Option<Uuid>,
    pub payroll_payable: Option<Uuid>,
    // Optional role_key → account_uuid overrides.
    // pub overrides: Option<std::collections::HashMap<String, Uuid>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payslip {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub payrun_uuid: Uuid,
    pub employee_uuid: Uuid,
    pub gross_pay: BigDecimal,
    pub basic_pay: Option<BigDecimal>,
    pub allowances_total: Option<BigDecimal>,
    pub tax_deducted: Option<BigDecimal>,
    pub social_security: Option<BigDecimal>,
    pub other_deductions: Option<BigDecimal>,
    pub employer_social_security: Option<BigDecimal>,
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
    pub other_deductions: Option<BigDecimal>,
    pub employer_social_security: Option<BigDecimal>,
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

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct EligibleEmployeeRow {
    pub employee_uuid: Uuid,
    pub bank_account_number: Option<String>,
    pub basic_salary: Option<BigDecimal>,
    pub payment_method: Option<String>,
}
