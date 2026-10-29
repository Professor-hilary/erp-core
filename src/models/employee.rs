//! HR + Payroll models & DTOs (process-aligned).
//! One module: people, employment, leave, lifecycle, payruns.

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =============================================================================
// ORGANISATION
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Department {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub parent_uuid: Option<Uuid>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartment {
    pub name: String,
    pub description: Option<String>,
    pub parent_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct JobTitle {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobTitle {
    pub title: String,
    pub description: Option<String>,
}

// =============================================================================
// EMPLOYEE (lean)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Employee {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub hire_date: NaiveDate,
    pub department_uuid: Option<Uuid>,
    pub job_title_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub party_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployee {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub hire_date: NaiveDate,
    pub department_uuid: Option<Uuid>,
    pub job_title_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub department_uuid: Option<Uuid>,
    pub job_title_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
    // status is NOT patched here — use lifecycle endpoints
}

#[derive(Debug, Clone, Serialize)]
pub struct SupervisorBrief {
    pub uuid: Uuid,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmployeeResponse {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub hire_date: NaiveDate,
    pub department: Option<Department>,
    pub job_title: Option<JobTitle>,
    pub supervisor: Option<SupervisorBrief>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub party_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// PROFILES + LIFECYCLE DATES
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeeStatutoryProfile {
    pub employee_uuid: Uuid,
    pub national_id: Option<String>,
    pub tax_id: Option<String>,
    pub social_security_number: Option<String>,
    pub other_ids: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertStatutoryProfile {
    pub national_id: Option<String>,
    pub tax_id: Option<String>,
    pub social_security_number: Option<String>,
    pub other_ids: Option<serde_json::Value>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeePaymentProfile {
    pub employee_uuid: Uuid,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub payment_method: String,
    pub currency_code: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertPaymentProfile {
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub payment_method: Option<String>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmployeeLifecycle {
    pub employee_uuid: Uuid,
    pub notice_start_date: Option<NaiveDate>,
    pub last_working_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub exit_reason_code: Option<String>,
    pub exit_notes: Option<String>,
    pub eligible_for_rehire: bool,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// CONTRACTS
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmploymentContract {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub employee_uuid: Uuid,
    pub contract_code: Option<String>,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub job_title_uuid: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub basic_salary: Option<BigDecimal>,
    pub pay_frequency: Option<String>,
    pub employment_type: Option<String>,
    pub probation_end_date: Option<NaiveDate>,
    pub notice_days: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmploymentContract {
    pub employee_uuid: Uuid,
    pub contract_code: Option<String>,
    pub contract_type: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub job_title_uuid: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub basic_salary: Option<BigDecimal>,
    pub pay_frequency: Option<String>,
    pub employment_type: Option<String>,
    pub probation_end_date: Option<NaiveDate>,
    pub notice_days: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ContractAmendment {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub contract_uuid: Uuid,
    pub effective_date: NaiveDate,
    pub amendment_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub basic_salary: Option<BigDecimal>,
    pub job_title_uuid: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractAmendment {
    pub effective_date: NaiveDate,
    pub amendment_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub basic_salary: Option<BigDecimal>,
    pub job_title_uuid: Option<Uuid>,
    pub notes: Option<String>,
}

// =============================================================================
// EMPLOYMENT EVENTS
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmploymentEvent {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub employee_uuid: Uuid,
    pub event_type: String,
    pub event_date: NaiveDate,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub reason_code: Option<String>,
    pub notes: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// LEAVE
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LeaveType {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub is_paid: bool,
    pub grant_mode: Option<String>,
    pub unused_treatment: Option<String>,
    pub default_days_per_year: Option<BigDecimal>,
    pub requires_approval: Option<bool>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveType {
    pub code: String,
    pub name: String,
    pub is_paid: Option<bool>,
    pub grant_mode: Option<String>,
    pub unused_treatment: Option<String>,
    pub default_days_per_year: Option<BigDecimal>,
    pub requires_approval: Option<bool>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LeaveBalance {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub entitled: BigDecimal,
    pub taken: BigDecimal,
    pub encashed: Option<BigDecimal>,
    pub balance: BigDecimal,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertLeaveBalance {
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub entitled: BigDecimal,
    pub taken: Option<BigDecimal>,
    pub encashed: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LeaveRequest {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: BigDecimal,
    pub status: String,
    pub reason: Option<String>,
    pub approver_uuid: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveRequest {
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub days: BigDecimal,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecideLeaveRequest {
    pub status: String, // Approved | Rejected
    pub approver_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LeaveSchedule {
    pub uuid: Uuid,
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub planned_start: NaiveDate,
    pub planned_end: NaiveDate,
    pub days_planned: BigDecimal,
    pub status: String,
    pub source: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveSchedule {
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub planned_start: NaiveDate,
    pub planned_end: NaiveDate,
    pub days_planned: BigDecimal,
    pub source: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveScheduleStatus {
    pub status: String, // Planned | Taken | Skipped | Encash_Pending | Encashed | Cancelled
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LeaveEncashment {
    pub uuid: Uuid,
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub days: BigDecimal,
    pub rate_per_day: BigDecimal,
    pub amount: BigDecimal,
    pub schedule_uuid: Option<Uuid>,
    pub payrun_uuid: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveEncashment {
    pub employee_uuid: Uuid,
    pub leave_type_uuid: Uuid,
    pub year: i32,
    pub days: BigDecimal,
    pub rate_per_day: BigDecimal,
    pub amount: BigDecimal,
    pub schedule_uuid: Option<Uuid>,
}

// =============================================================================
// ONBOARDING / OFFBOARDING TASKS
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OnboardingTask {
    pub uuid: Uuid,
    pub serial_id: Option<i64>,
    pub employee_uuid: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub is_required: bool,
    pub is_done: bool,
    pub due_date: Option<NaiveDate>,
    pub done_at: Option<DateTime<Utc>>,
    pub done_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub type OffboardingTask = OnboardingTask; // same shape if tables match

#[derive(Debug, Deserialize)]
pub struct CompleteChecklistTask {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartExit {
    pub last_working_date: NaiveDate,
    pub exit_reason_code: String,
    pub notice_start_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

// =============================================================================
// PAYROLL SNAPSHOT (read model for generate payslips — optional FromRow)
// =============================================================================

#[derive(Debug, Clone, FromRow, Serialize)]
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
}

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
    /// Optional role_key → account_uuid overrides.
    pub overrides: Option<std::collections::HashMap<String, Uuid>>,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct CreatePayslipItem {
    pub item_type: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub component_uuid: Option<Uuid>,
}

/*
// src/models/employee.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Employee {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
}

//---------------------//-----------------//----------------
pub struct Employee {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub hire_date: NaiveDate,
    pub department_uuid: Option<Uuid>,
    pub job_title_uuid: Option<Uuid>,  // was job_title
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub party_uuid: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CreateEmployee {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub email: Option<String>,
    pub work_email: Option<String>,
    pub phone_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub gender: Option<String>,
    pub hire_date: NaiveDate,
    pub department_uuid: Option<Uuid>,
    pub job_title_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub pay_frequency: Option<String>,
    // status optional; DB default Onboarding
}
//----------------//----------------//----------------

#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<JobTitle>,
    pub department: Option<Department>,
    pub supervisor: Option<SupervisorResponse>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Department {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartment {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SupervisorResponse {
    pub uuid: Uuid,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct JobTitle {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobTitle {
    pub title: String,
    pub description: Option<String>,
}
*/
