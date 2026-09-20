use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentProgram {
    pub id: Uuid,
    pub serial_id: i64,
    pub company_id: Uuid,
    pub program_code: String,
    pub name: String,
    pub description: Option<String>,
    pub horizon_start: Option<NaiveDate>,
    pub horizon_end: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvestmentProgram {
    #[validate(length(min = 1, max = 64))]
    pub program_code: String,
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    pub description: Option<String>,
    pub horizon_start: Option<NaiveDate>,
    pub horizon_end: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Cases
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentCase {
    pub id: Uuid,
    pub serial_id: i64,
    pub company_id: Uuid,
    pub case_code: String,
    pub title: String,
    pub description: Option<String>,
    pub investment_type: String,
    pub program_id: Option<Uuid>,
    pub strategic_score: Option<BigDecimal>,
    pub strategic_notes: Option<String>,
    pub risk_rating: Option<String>,
    pub stage: String,
    pub stage_changed_at: DateTime<Utc>,
    pub project_id: Option<Uuid>,
    pub currency_id: Uuid,
    pub hurdle_rate: Option<BigDecimal>,
    pub wacc_as_of: Option<NaiveDate>,
    pub npv: Option<BigDecimal>,
    pub irr: Option<BigDecimal>,
    pub mirr: Option<BigDecimal>,
    pub payback_years: Option<BigDecimal>,
    pub discounted_payback: Option<BigDecimal>,
    pub profitability_index: Option<BigDecimal>,
    pub arr: Option<BigDecimal>,
    pub roce: Option<BigDecimal>,
    pub metrics_calculated_at: Option<DateTime<Utc>>,
    pub actual_irr: Option<BigDecimal>,
    pub actual_npv: Option<BigDecimal>,
    pub audit_completed_at: Option<NaiveDate>,
    pub lessons_learned: Option<String>,
    pub initiator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvestmentCase {
    #[validate(length(min = 1, max = 64))]
    pub case_code: String,
    #[validate(length(min = 1, max = 256))]
    pub title: String,
    pub description: Option<String>,
    pub investment_type: String, // REPLACEMENT | EXPANSION | ...
    pub program_id: Option<Uuid>,
    pub strategic_score: Option<BigDecimal>,
    pub strategic_notes: Option<String>,
    pub risk_rating: Option<String>,
    pub currency_id: Uuid,
    pub hurdle_rate: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateInvestmentCase {
    pub title: Option<String>,
    pub description: Option<String>,
    pub investment_type: Option<String>,
    pub program_id: Option<Uuid>,
    pub strategic_score: Option<BigDecimal>,
    pub strategic_notes: Option<String>,
    pub risk_rating: Option<String>,
    pub hurdle_rate: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AdvanceStageRequest {
    pub to_stage: String,
    pub comment: Option<String>,
}

// ---------------------------------------------------------------------------
// Scenarios & cash flows
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentScenario {
    pub id: Uuid,
    pub case_id: Uuid,
    pub scenario_code: String,
    pub name: String,
    pub probability: Option<BigDecimal>,
    pub inflation_rate: Option<BigDecimal>,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvestmentScenario {
    pub scenario_code: String, // BASE | BEST | WORST | CUSTOM
    pub name: String,
    pub probability: Option<BigDecimal>,
    pub inflation_rate: Option<BigDecimal>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentCashflowLine {
    pub id: Uuid,
    pub serial_id: i64,
    pub scenario_id: Uuid,
    pub period_no: i32,
    pub period_date: Option<NaiveDate>,
    pub line_type: String,
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub is_cash: bool,
    pub tax_deductible: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCashflowLine {
    pub period_no: i32,
    pub period_date: Option<NaiveDate>,
    pub line_type: String, // CAPEX | OPEX | REVENUE | ...
    pub description: Option<String>,
    pub amount: BigDecimal,
    pub is_cash: Option<bool>,
    pub tax_deductible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentMetrics {
    pub id: Uuid,
    pub scenario_id: Uuid,
    pub discount_rate: BigDecimal,
    pub npv: Option<BigDecimal>,
    pub irr: Option<BigDecimal>,
    pub mirr: Option<BigDecimal>,
    pub payback_years: Option<BigDecimal>,
    pub discounted_payback: Option<BigDecimal>,
    pub profitability_index: Option<BigDecimal>,
    pub arr: Option<BigDecimal>,
    pub roce: Option<BigDecimal>,
    pub total_capex: Option<BigDecimal>,
    pub total_pv_inflows: Option<BigDecimal>,
    pub calculated_at: DateTime<Utc>,
    pub calculated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalculateMetricsRequest {
    /// If omitted, uses case.hurdle_rate
    pub discount_rate: Option<BigDecimal>,
    /// Finance rate for MIRR (reinvestment); default = discount_rate
    pub reinvest_rate: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PeriodCashflow {
    pub period_no: i32,
    pub net_cash: BigDecimal,
}

// ---------------------------------------------------------------------------
// Risks, sensitivity, post-audit (thin DTOs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentRisk {
    pub id: Uuid,
    pub case_id: Uuid,
    pub risk_code: Option<String>,
    pub title: String,
    pub category: Option<String>,
    pub likelihood: Option<String>,
    pub impact: Option<String>,
    pub score: Option<i32>,
    pub mitigation: Option<String>,
    pub owner_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInvestmentRisk {
    pub risk_code: Option<String>,
    pub title: String,
    pub category: Option<String>,
    pub likelihood: Option<String>,
    pub impact: Option<String>,
    pub mitigation: Option<String>,
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePostAudit {
    pub audit_date: NaiveDate,
    pub actual_npv: Option<BigDecimal>,
    pub actual_irr: Option<BigDecimal>,
    pub actual_payback: Option<BigDecimal>,
    pub variance_notes: Option<String>,
    pub lessons_learned: Option<String>,
    pub performance_link: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InvestmentPostAudit {
    pub id: Uuid,
    pub case_id: Uuid,
    pub audit_date: NaiveDate,
    pub promised_npv: Option<BigDecimal>,
    pub promised_irr: Option<BigDecimal>,
    pub actual_npv: Option<BigDecimal>,
    pub actual_irr: Option<BigDecimal>,
    pub actual_payback: Option<BigDecimal>,
    pub variance_notes: Option<String>,
    pub lessons_learned: Option<String>,
    pub performance_link: Option<serde_json::Value>,
    pub audited_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioBoardRow {
    pub case_id: Uuid,
    pub case_code: String,
    pub title: String,
    pub investment_type: String,
    pub stage: String,
    pub risk_rating: Option<String>,
    pub strategic_score: Option<BigDecimal>,
    pub npv: Option<BigDecimal>,
    pub irr: Option<BigDecimal>,
    pub payback_years: Option<BigDecimal>,
    pub program_code: Option<String>,
    pub program_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub approved_budget: Option<BigDecimal>,
    pub spent_to_date: Option<BigDecimal>,
    pub budget_burn_pct: Option<BigDecimal>,
}

