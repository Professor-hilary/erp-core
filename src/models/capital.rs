// src/models/capital/mod.rs  (or split into multiple files)

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// 1. CAPITAL INSTRUMENTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalInstrument {
    pub id: Uuid,
    pub company_id: Uuid,
    pub instrument_code: String,
    pub name: String,
    pub instrument_family: String, // DEBT | EQUITY | HYBRID | MEZZANINE | OTHER
    pub instrument_type: String,
    pub currency_id: Uuid,
    pub original_principal: BigDecimal,
    pub outstanding_principal: BigDecimal,
    pub face_value: Option<BigDecimal>,
    pub issue_price: Option<BigDecimal>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub is_perpetual: bool,
    pub ranking: Option<String>,
    pub is_callable: bool,
    pub is_putable: bool,
    pub is_convertible: bool,
    pub conversion_ratio: Option<BigDecimal>,
    pub conversion_price: Option<BigDecimal>,
    pub status: String,
    pub accounting_treatment: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalInstrument {
    #[validate(length(min = 1, max = 64))]
    pub instrument_code: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub instrument_family: String,
    pub instrument_type: String,
    pub currency_id: Uuid,
    pub original_principal: BigDecimal,
    pub face_value: Option<BigDecimal>,
    pub issue_price: Option<BigDecimal>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub is_perpetual: Option<bool>,
    pub ranking: Option<String>,
    pub is_callable: Option<bool>,
    pub is_putable: Option<bool>,
    pub is_convertible: Option<bool>,
    pub conversion_ratio: Option<BigDecimal>,
    pub conversion_price: Option<BigDecimal>,
    pub status: Option<String>,
    pub accounting_treatment: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCapitalInstrument {
    pub name: Option<String>,
    pub outstanding_principal: Option<BigDecimal>,
    pub status: Option<String>,
    pub maturity_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

// =============================================================================
// 2. CAPITAL EVENTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalEvent {
    pub id: Uuid,
    pub company_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub event_type: String,
    pub event_date: NaiveDate,
    pub effective_date: Option<NaiveDate>,
    pub amount: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub shares: Option<i64>,
    pub description: Option<String>,
    pub related_party_id: Option<Uuid>,
    pub journal_entry_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalEvent {
    pub instrument_id: Option<Uuid>,
    pub event_type: String,
    pub event_date: NaiveDate,
    pub effective_date: Option<NaiveDate>,
    pub amount: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub shares: Option<i64>,
    pub description: Option<String>,
    pub related_party_id: Option<Uuid>,
    pub journal_entry_id: Option<Uuid>,
}

// =============================================================================
// 3. DEBT FACILITIES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalFacility {
    pub id: Uuid,
    pub company_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub facility_code: String,
    pub facility_name: String,
    pub facility_type: String,
    pub agent_id: Option<Uuid>,
    pub currency_id: Uuid,
    pub committed_amount: BigDecimal,
    pub available_amount: BigDecimal,
    pub drawn_amount: BigDecimal,
    pub interest_type: Option<String>,
    pub base_rate_index: Option<String>,
    pub margin_bps: Option<BigDecimal>,
    pub floor_rate: Option<BigDecimal>,
    pub ceiling_rate: Option<BigDecimal>,
    pub day_count_convention: Option<String>,
    pub payment_frequency: Option<String>,
    pub amortization_type: Option<String>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub commitment_fee_bps: Option<BigDecimal>,
    pub utilization_fee_bps: Option<BigDecimal>,
    pub prepayment_penalty: Option<String>,
    pub collateral_required: bool,
    pub is_secured: bool,
    pub ranking: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalFacility {
    pub instrument_id: Option<Uuid>,
    #[validate(length(min = 1, max = 64))]
    pub facility_code: String,
    #[validate(length(min = 1, max = 255))]
    pub facility_name: String,
    pub facility_type: String,
    pub agent_id: Option<Uuid>,
    pub currency_id: Uuid,
    pub committed_amount: BigDecimal,
    pub interest_type: Option<String>,
    pub base_rate_index: Option<String>,
    pub margin_bps: Option<BigDecimal>,
    pub floor_rate: Option<BigDecimal>,
    pub ceiling_rate: Option<BigDecimal>,
    pub day_count_convention: Option<String>,
    pub payment_frequency: Option<String>,
    pub amortization_type: Option<String>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub commitment_fee_bps: Option<BigDecimal>,
    pub utilization_fee_bps: Option<BigDecimal>,
    pub prepayment_penalty: Option<String>,
    pub collateral_required: Option<bool>,
    pub is_secured: Option<bool>,
    pub ranking: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCapitalFacility {
    pub facility_name: Option<String>,
    pub available_amount: Option<BigDecimal>,
    pub drawn_amount: Option<BigDecimal>,
    pub margin_bps: Option<BigDecimal>,
    pub maturity_date: Option<NaiveDate>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FacilityLender {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub lender_id: Uuid,
    pub commitment_amount: BigDecimal,
    pub participation_pct: Option<BigDecimal>,
    pub is_agent: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// 4. DRAWDOWNS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtDrawdown {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub drawdown_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: BigDecimal,
    pub currency_id: Uuid,
    pub reference: Option<String>,
    pub purpose: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDrawdown {
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub drawdown_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: BigDecimal,
    pub currency_id: Uuid,
    pub reference: Option<String>,
    pub purpose: Option<String>,
    pub status: Option<String>,
}

// =============================================================================
// 5. REPAYMENT SCHEDULES & REPAYMENTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtRepaymentSchedule {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub sequence_no: i32,
    pub due_date: NaiveDate,
    pub principal_due: BigDecimal,
    pub interest_due: BigDecimal,
    pub fee_due: BigDecimal,
    pub total_due: BigDecimal, // generated column
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRepaymentSchedule {
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub sequence_no: i32,
    pub due_date: NaiveDate,
    pub principal_due: BigDecimal,
    pub interest_due: BigDecimal,
    pub fee_due: Option<BigDecimal>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtRepayment {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub repayment_date: NaiveDate,
    pub principal_amount: BigDecimal,
    pub interest_amount: BigDecimal,
    pub fee_amount: BigDecimal,
    pub total_amount: BigDecimal, // generated
    pub currency_id: Uuid,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRepayment {
    pub facility_id: Uuid,
    pub schedule_id: Option<Uuid>,
    pub repayment_date: NaiveDate,
    pub principal_amount: BigDecimal,
    pub interest_amount: BigDecimal,
    pub fee_amount: Option<BigDecimal>,
    pub currency_id: Uuid,
    pub payment_method: Option<String>,
    pub reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
}

// =============================================================================
// 6. INTEREST ACCRUALS & FEES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtInterestAccrual {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub principal_base: BigDecimal,
    pub annual_rate: BigDecimal,
    pub day_count: i32,
    pub interest_amount: BigDecimal,
    pub is_paid: bool,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInterestAccrual {
    pub facility_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub principal_base: BigDecimal,
    pub annual_rate: BigDecimal,
    pub day_count: i32,
    pub interest_amount: BigDecimal,
    pub is_paid: Option<bool>,
    pub journal_entry_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtFee {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub fee_type: String,
    pub fee_date: NaiveDate,
    pub amount: BigDecimal,
    pub currency_id: Uuid,
    pub description: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDebtFee {
    pub facility_id: Uuid,
    pub fee_type: String,
    pub fee_date: NaiveDate,
    pub amount: BigDecimal,
    pub currency_id: Uuid,
    pub description: Option<String>,
    pub journal_entry_id: Option<Uuid>,
}

// =============================================================================
// 7. COVENANTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtCovenant {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub covenant_code: String,
    pub covenant_name: String,
    pub covenant_type: String,
    pub metric: Option<String>,
    pub operator: Option<String>,
    pub threshold_min: Option<BigDecimal>,
    pub threshold_max: Option<BigDecimal>,
    pub measurement_frequency: Option<String>,
    pub testing_date_rule: Option<String>,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub cure_period_days: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDebtCovenant {
    pub facility_id: Uuid,
    pub covenant_code: String,
    pub covenant_name: String,
    pub covenant_type: String,
    pub metric: Option<String>,
    pub operator: Option<String>,
    pub threshold_min: Option<BigDecimal>,
    pub threshold_max: Option<BigDecimal>,
    pub measurement_frequency: Option<String>,
    pub testing_date_rule: Option<String>,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub cure_period_days: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtCovenantTest {
    pub id: Uuid,
    pub covenant_id: Uuid,
    pub test_date: NaiveDate,
    pub actual_value: Option<BigDecimal>,
    pub is_compliant: Option<bool>,
    pub headroom: Option<BigDecimal>,
    pub notes: String,
    pub tested_by: Uuid,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// 8. COLLATERAL & REFINANCING
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtCollateral {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub collateral_type: String,
    pub description: Option<String>,
    pub estimated_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub valuation_date: Option<NaiveDate>,
    pub ranking: Option<String>,
    pub is_perfected: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDebtCollateral {
    pub facility_id: Uuid,
    pub collateral_type: String,
    pub description: Option<String>,
    pub estimated_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub valuation_date: Option<NaiveDate>,
    pub ranking: Option<String>,
    pub is_perfected: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DebtRefinancing {
    pub id: Uuid,
    pub company_id: Uuid,
    pub old_facility_id: Uuid,
    pub new_facility_id: Option<Uuid>,
    pub refinancing_date: NaiveDate,
    pub principal_refinanced: BigDecimal,
    pub costs: BigDecimal,
    pub description: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDebtRefinancing {
    pub old_facility_id: Uuid,
    pub new_facility_id: Option<Uuid>,
    pub refinancing_date: NaiveDate,
    pub principal_refinanced: BigDecimal,
    pub costs: Option<BigDecimal>,
    pub description: Option<String>,
    pub journal_entry_id: Option<Uuid>,
}

// =============================================================================
// 9. EQUITY – SHARE CLASSES & SHAREHOLDERS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShareClass {
    pub id: Uuid,
    pub company_id: Uuid,
    pub class_code: String,
    pub name: String,
    pub share_type: String,
    pub authorized_shares: Option<i64>,
    pub issued_shares: i64,
    pub outstanding_shares: i64,
    pub par_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub voting_rights: bool,
    pub votes_per_share: Option<BigDecimal>,
    pub dividend_rights: bool,
    pub dividend_preference: Option<String>,
    pub liquidation_preference: Option<BigDecimal>,
    pub is_callable: bool,
    pub is_convertible: bool,
    pub conversion_ratio: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareClass {
    pub class_code: String,
    pub name: String,
    pub share_type: String,
    pub authorized_shares: Option<i64>,
    pub par_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub voting_rights: Option<bool>,
    pub votes_per_share: Option<BigDecimal>,
    pub dividend_rights: Option<bool>,
    pub dividend_preference: Option<String>,
    pub liquidation_preference: Option<BigDecimal>,
    pub is_callable: Option<bool>,
    pub is_convertible: Option<bool>,
    pub conversion_ratio: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateShareClass {
    pub name: Option<String>,
    pub authorized_shares: Option<i64>,
    pub voting_rights: Option<bool>,
    pub dividend_rights: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Shareholder {
    pub id: Uuid,
    pub company_id: Uuid,
    pub party_id: Option<Uuid>,
    pub name: String,
    pub shareholder_type: String,
    pub tax_id: Option<String>,
    pub residency_country: Option<String>,
    pub is_related_party: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareholder {
    pub party_id: Option<Uuid>,
    pub name: String,
    pub shareholder_type: String,
    pub tax_id: Option<String>,
    pub residency_country: Option<String>,
    pub is_related_party: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Shareholding {
    pub id: Uuid,
    pub company_id: Uuid,
    pub share_class_id: Uuid,
    pub shareholder_id: Uuid,
    pub shares_held: i64,
    pub average_cost: Option<BigDecimal>,
    pub acquisition_date: Option<NaiveDate>,
    pub is_beneficial: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// 10. SHARE TRANSACTIONS & DIVIDENDS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ShareTransaction {
    pub id: Uuid,
    pub company_id: Uuid,
    pub share_class_id: Uuid,
    pub transaction_type: String,
    pub transaction_date: NaiveDate,
    pub from_shareholder_id: Option<Uuid>,
    pub to_shareholder_id: Option<Uuid>,
    pub shares: i64,
    pub price_per_share: Option<BigDecimal>,
    pub total_consideration: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub premium: Option<BigDecimal>,
    pub journal_entry_id: Option<Uuid>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateShareTransaction {
    pub share_class_id: Uuid,
    pub transaction_type: String,
    pub transaction_date: NaiveDate,
    pub from_shareholder_id: Option<Uuid>,
    pub to_shareholder_id: Option<Uuid>,
    pub shares: i64,
    pub price_per_share: Option<BigDecimal>,
    pub total_consideration: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub premium: Option<BigDecimal>,
    pub journal_entry_id: Option<Uuid>,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Dividend {
    pub id: Uuid,
    pub company_id: Uuid,
    pub share_class_id: Uuid,
    pub dividend_type: String,
    pub declaration_date: NaiveDate,
    pub record_date: Option<NaiveDate>,
    pub ex_dividend_date: Option<NaiveDate>,
    pub payment_date: Option<NaiveDate>,
    pub dividend_per_share: BigDecimal,
    pub total_declared: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub status: String,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateDividend {
    pub share_class_id: Uuid,
    pub dividend_type: String,
    pub declaration_date: NaiveDate,
    pub record_date: Option<NaiveDate>,
    pub ex_dividend_date: Option<NaiveDate>,
    pub payment_date: Option<NaiveDate>,
    pub dividend_per_share: BigDecimal,
    pub total_declared: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DividendPayment {
    pub id: Uuid,
    pub dividend_id: Uuid,
    pub shareholder_id: Uuid,
    pub shares_held: i64,
    pub gross_amount: BigDecimal,
    pub withholding_tax: BigDecimal,
    pub net_amount: BigDecimal,
    pub payment_date: Option<NaiveDate>,
    pub payment_reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// 11. EQUITY ACCOUNTS & MOVEMENTS (Retained Earnings etc.)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EquityAccount {
    pub id: Uuid,
    pub company_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String, // SHARE_CAPITAL, RETAINED_EARNINGS, ...
    pub currency_id: Option<Uuid>,
    pub is_distributable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEquityAccount {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub currency_id: Option<Uuid>,
    pub is_distributable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EquityMovement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub equity_account_id: Uuid,
    pub movement_date: NaiveDate,
    pub movement_type: String,
    pub amount: BigDecimal,
    pub description: Option<String>,
    pub related_instrument_id: Option<Uuid>,
    pub related_event_id: Option<Uuid>,
    pub journal_entry_id: Option<Uuid>,
    pub period_year: Option<i32>,
    pub period_month: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateEquityMovement {
    pub equity_account_id: Uuid,
    pub movement_date: NaiveDate,
    pub movement_type: String,
    pub amount: BigDecimal,
    pub description: Option<String>,
    pub related_instrument_id: Option<Uuid>,
    pub related_event_id: Option<Uuid>,
    pub journal_entry_id: Option<Uuid>,
    pub period_year: Option<i32>,
    pub period_month: Option<i32>,
}

// =============================================================================
// 12. CAPITAL STRUCTURE & ALLOCATIONS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalStructureSnapshot {
    pub id: Uuid,
    pub company_id: Uuid,
    pub as_of_date: NaiveDate,
    pub total_debt: BigDecimal,
    pub total_equity: BigDecimal,
    pub total_hybrid: BigDecimal,
    pub cash_and_equivalents: Option<BigDecimal>,
    pub net_debt: BigDecimal, // generated
    pub debt_to_equity: Option<BigDecimal>,
    pub equity_ratio: Option<BigDecimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalStructureSnapshot {
    pub as_of_date: NaiveDate,
    pub total_debt: BigDecimal,
    pub total_equity: BigDecimal,
    pub total_hybrid: Option<BigDecimal>,
    pub cash_and_equivalents: Option<BigDecimal>,
    pub debt_to_equity: Option<BigDecimal>,
    pub equity_ratio: Option<BigDecimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalAllocation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub allocation_code: Option<String>,
    pub capital_source_type: String,
    pub capital_source_id: Uuid,
    pub allocation_type: String,
    pub allocation_target_id: Option<Uuid>,
    pub amount: BigDecimal,
    pub currency_id: Option<Uuid>,
    pub allocation_date: NaiveDate,
    pub expected_return: Option<BigDecimal>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalAllocation {
    pub allocation_code: Option<String>,
    pub capital_source_type: String,
    pub capital_source_id: Uuid,
    pub allocation_type: String,
    pub allocation_target_id: Option<Uuid>,
    pub amount: BigDecimal,
    pub currency_id: Option<Uuid>,
    pub allocation_date: NaiveDate,
    pub expected_return: Option<BigDecimal>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

// =============================================================================
// 13. PROJECTS & DEPLOYMENT
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalProject {
    pub id: Uuid,
    pub company_id: Uuid,
    pub project_code: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub expected_completion: Option<NaiveDate>,
    pub actual_completion: Option<NaiveDate>,
    pub approved_budget: Option<BigDecimal>,
    pub spent_to_date: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub expected_irr: Option<BigDecimal>,
    pub expected_npv: Option<BigDecimal>,
    pub expected_payback_years: Option<BigDecimal>,
    pub risk_rating: Option<String>,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalProject {
    pub project_code: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub expected_completion: Option<NaiveDate>,
    pub approved_budget: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub expected_irr: Option<BigDecimal>,
    pub expected_npv: Option<BigDecimal>,
    pub expected_payback_years: Option<BigDecimal>,
    pub risk_rating: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCapitalProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub spent_to_date: Option<BigDecimal>,
    pub actual_completion: Option<NaiveDate>,
    pub expected_irr: Option<BigDecimal>,
    pub expected_npv: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectFunding {
    pub id: Uuid,
    pub project_id: Uuid,
    pub funding_source_type: String,
    pub funding_source_id: Uuid,
    pub amount_committed: BigDecimal,
    pub amount_drawn: BigDecimal,
    pub currency_id: Option<Uuid>,
    pub funding_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProjectFunding {
    pub project_id: Uuid,
    pub funding_source_type: String,
    pub funding_source_id: Uuid,
    pub amount_committed: BigDecimal,
    pub amount_drawn: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub funding_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentPosition {
    pub id: Uuid,
    pub company_id: Uuid,
    pub position_name: String,
    pub asset_class: Option<String>,
    pub instrument_id: Option<Uuid>,
    pub quantity: Option<BigDecimal>,
    pub cost_basis: Option<BigDecimal>,
    pub current_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub valuation_date: Option<NaiveDate>,
    pub unrealized_pnl: Option<BigDecimal>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvestmentPosition {
    pub position_name: String,
    pub asset_class: Option<String>,
    pub instrument_id: Option<Uuid>,
    pub quantity: Option<BigDecimal>,
    pub cost_basis: Option<BigDecimal>,
    pub current_value: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub valuation_date: Option<NaiveDate>,
    pub status: Option<String>,
}

// =============================================================================
// 14. LIQUIDITY / CASH FORECASTS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashForecast {
    pub id: Uuid,
    pub company_id: Uuid,
    pub forecast_name: Option<String>,
    pub forecast_date: NaiveDate,
    pub horizon_days: i32,
    pub currency_id: Option<Uuid>,
    pub opening_cash: Option<BigDecimal>,
    pub scenario: String,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCashForecast {
    pub forecast_name: Option<String>,
    pub forecast_date: NaiveDate,
    pub horizon_days: Option<i32>,
    pub currency_id: Option<Uuid>,
    pub opening_cash: Option<BigDecimal>,
    pub scenario: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashForecastLine {
    pub id: Uuid,
    pub forecast_id: Uuid,
    pub line_date: NaiveDate,
    pub category: String,
    pub description: Option<String>,
    pub amount: BigDecimal, // signed
    pub is_committed: bool,
    pub related_facility_id: Option<Uuid>,
    pub related_project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCashForecastLine {
    pub line_date: NaiveDate,
    pub category: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub is_committed: Option<bool>,
    pub related_facility_id: Option<Uuid>,
    pub related_project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FundingRequirement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub requirement_date: NaiveDate,
    pub amount_needed: BigDecimal,
    pub currency_id: Option<Uuid>,
    pub purpose: Option<String>,
    pub preferred_source: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateFundingRequirement {
    pub requirement_date: NaiveDate,
    pub amount_needed: BigDecimal,
    pub currency_id: Option<Uuid>,
    pub purpose: Option<String>,
    pub preferred_source: Option<String>,
    pub status: Option<String>,
}

// =============================================================================
// 15. ANALYTICS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CapitalMetric {
    pub id: Uuid,
    pub company_id: Uuid,
    pub metric_date: NaiveDate,
    pub metric_code: String, // ROIC, ROE, DSCR, ICR, EVA, WACC...
    pub metric_name: String,
    pub value: BigDecimal,
    pub numerator: Option<BigDecimal>,
    pub denominator: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub period_type: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCapitalMetric {
    pub metric_date: NaiveDate,
    pub metric_code: String,
    pub metric_name: String,
    pub value: BigDecimal,
    pub numerator: Option<BigDecimal>,
    pub denominator: Option<BigDecimal>,
    pub currency_id: Option<Uuid>,
    pub period_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WaccComponent {
    pub id: Uuid,
    pub company_id: Uuid,
    pub as_of_date: NaiveDate,
    pub cost_of_equity: Option<BigDecimal>,
    pub cost_of_debt: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub equity_weight: Option<BigDecimal>,
    pub debt_weight: Option<BigDecimal>,
    pub wacc: Option<BigDecimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateWaccComponent {
    pub as_of_date: NaiveDate,
    pub cost_of_equity: Option<BigDecimal>,
    pub cost_of_debt: Option<BigDecimal>,
    pub tax_rate: Option<BigDecimal>,
    pub equity_weight: Option<BigDecimal>,
    pub debt_weight: Option<BigDecimal>,
    pub wacc: Option<BigDecimal>,
    pub notes: Option<String>,
}
