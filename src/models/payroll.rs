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
	pub sequence_no: i16,
	pub rounding_scale: i16,
	pub effective_from: Option<NaiveDate>,
	pub effective_to: Option<NaiveDate>,
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
	#[serde(default = "default_component_sequence")]
	pub sequence_no: i16,

	#[serde(default = "default_rounding_scale")]
	pub rounding_scale: i16,

	pub effective_from: Option<NaiveDate>,
	pub effective_to: Option<NaiveDate>,
    pub is_taxable: Option<bool>,
    pub affects_social_security: Option<bool>,
    pub statutory_role: Option<String>,
    pub gl_account_hint: Option<String>,
}

fn default_component_sequence() -> i16 {
    100
}

fn default_rounding_scale() -> i16 {
    2
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

/*#[derive(Debug, Deserialize)]
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
}*/

#[derive(Debug, Deserialize)]
pub struct PostPayrun {
    pub payrun_serial_id: i64,
    pub labor_expense_id: Option<Uuid>,
    pub income_tax_id: Option<Uuid>,
    pub employee_social_security_id: Option<Uuid>,
    pub employer_social_security_id: Option<Uuid>,
    pub employer_social_security_expense_id: Option<Uuid>,
    pub payroll_payable: Option<Uuid>,
    /// Required when any payslip has other_deductions > 0
    pub other_deductions_payable_id: Option<Uuid>,
    /// Required when any payslip has employer costs beyond NSSF
    pub other_employer_cost_expense_id: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payslip {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub payrun_uuid: Uuid,
    pub employee_uuid: Uuid,
    pub gross_pay: BigDecimal,
    pub basic_pay: Option<BigDecimal>,
    pub allowances_total: BigDecimal,
    pub taxable_earnings: BigDecimal,
    pub tax_deducted: Option<BigDecimal>,
    pub social_security: Option<BigDecimal>,
    pub other_deductions: Option<BigDecimal>,
    pub employer_social_security: Option<BigDecimal>,
    pub employer_cost_total: BigDecimal,
    pub net_pay: Option<BigDecimal>,
    pub payment_method: Option<String>,
    pub bank_account: Option<String>,
    pub status: Option<String>,
    pub calculation_version: Option<String>,
    pub calculated_at: Option<DateTime<Utc>>,
    pub calculation_snapshot: Option<serde_json::Value>,
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

/// Full context required to calculate one employee in a payrun.
#[derive(Debug, Clone, FromRow)]
pub struct EmployeePayrollContext {
    pub employee_uuid: Uuid,
    pub date_of_birth: Option<NaiveDate>,
    pub tax_residency: String,
    pub nssf_covered: bool,
    pub nssf_exempt: bool,
    pub contract_uuid: Option<Uuid>,
    pub contract_start: Option<NaiveDate>,
    pub contract_end: Option<NaiveDate>,
    pub basic_salary: Option<BigDecimal>,
    pub pay_frequency: String,
    pub payment_method: Option<String>,
    pub bank_account_number: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ComponentAssignmentRow {
    pub component_uuid: Uuid,
    pub code: String,
    pub name: String,
    pub component_kind: String,
    pub calc_method: String,
    pub amount: Option<BigDecimal>,
    pub rate: Option<BigDecimal>,
    pub is_taxable: bool,
    pub affects_social_security: bool,
    pub sequence_no: i16,
    pub rounding_scale: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct StatutoryBandRow {
    pub lower_bound: BigDecimal,
    pub upper_bound: Option<BigDecimal>,
    pub base_tax: BigDecimal,
    pub marginal_rate: BigDecimal,
    pub sequence_no: i16,
}

#[derive(Debug, Clone, FromRow)]
pub struct NssfRuleRow {
    // pub rule_set_uuid: Uuid,
    pub employee_rate: BigDecimal,
    pub employer_rate: BigDecimal,
    pub minimum_age: Option<i16>,
    pub maximum_age: Option<i16>,
}

#[derive(Debug, Clone, FromRow)]
pub struct EmployeeTaxPeriodRow {
    pub uuid: Uuid,
    pub taxable_income: BigDecimal,
    pub paye_due: BigDecimal,
    pub paye_withheld: BigDecimal,
}

#[derive(Debug, Clone, FromRow)]
pub struct LeaveEncashmentRow {
    pub uuid: Uuid,
    pub amount: BigDecimal,
    pub days: BigDecimal,
    pub leave_type_code: Option<String>,
}

// =============================================================================
// COMPONENT ASSIGNMENTS
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeeComponentAssignment {
    pub uuid: Uuid,
    pub employee_uuid: Uuid,
    pub component_uuid: Uuid,
    pub amount: Option<BigDecimal>,
    pub rate: Option<BigDecimal>,
    pub calculation_basis: Option<String>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateComponentAssignment {
    pub employee_uuid: Uuid,
    pub component_uuid: Uuid,
    pub amount: Option<BigDecimal>,
    pub rate: Option<BigDecimal>,
    pub calculation_basis: Option<String>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}

// =============================================================================
// EMPLOYEE SETTLEMENT
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeePaymentBatch {
    pub uuid: Uuid,
    pub payrun_uuid: Uuid,
    pub payment_reference: Option<String>,
    pub payment_date: NaiveDate,
    pub status: String,
    pub gl_transaction_uuid: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeePaymentLine {
    pub uuid: Uuid,
    pub payment_batch_uuid: Uuid,
    pub payslip_uuid: Uuid,
    pub amount: BigDecimal,
    pub payment_reference: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SettleEmployees {
    pub payrun_uuid: Uuid,
    pub payment_date: NaiveDate,
    pub payment_reference: Option<String>,
    /// Bank / cash account to credit when settling net wages
    pub cash_account_uuid: Option<Uuid>,
    /// Optional — resolved from payroll_net_payable when omitted
    pub payroll_payable_uuid: Option<Uuid>,
}

// =============================================================================
// STATUTORY REMITTANCE
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StatutoryRemittance {
    pub uuid: Uuid,
    pub payrun_uuid: Uuid,
    pub statutory_code: String,
    pub amount: BigDecimal,
    pub due_date: NaiveDate,
    pub payment_date: Option<NaiveDate>,
    pub status: String,
    pub gl_transaction_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStatutoryRemittance {
    pub payrun_uuid: Uuid,
	/// PAYE | NSSF_EE | NSSF_ER — empty / "*" / "ALL" creates every non-zero liability
	#[serde(default)]
    pub statutory_code: String, // PAYE | NSSF_EE | NSSF_ER
    pub due_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct RemitStatutory {
    pub remittance_uuid: Uuid,
    pub payment_date: NaiveDate,
    /// Liability account being settled (PAYE payable / NSSF payable)
    pub liability_account_uuid: Uuid,
    /// Bank / cash account
    pub cash_account_uuid: Uuid,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayslipCalculationItem {
    pub uuid: Uuid,
    pub payslip_uuid: Uuid,
    pub component_uuid: Option<Uuid>,
    pub item_code: String,
    pub item_name: String,
    pub item_kind: String,
    pub calculation_method: String,
    pub basis_amount: BigDecimal,
    pub rate: Option<BigDecimal>,
    pub quantity: BigDecimal,
    pub unrounded_amount: BigDecimal,
    pub amount: BigDecimal,
    pub taxable: bool,
    pub included_in_nssf_base: bool,
    pub created_at: DateTime<Utc>,
}
