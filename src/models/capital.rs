use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// DEBT FACILITY
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CapitalFacility {
    pub id: Uuid,
    pub company_id: Uuid,
    pub lender_id: Uuid,
    pub facility_name: String,
    pub facility_type: String,
    pub currency_id: Uuid,
    pub approved_limit: BigDecimal,
    pub interest_rate: Option<BigDecimal>,
    pub interest_type: Option<String>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub collateral_required: bool,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================
// CREATE DEBT FACILITY
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateCapitalFacility {
    pub lender_id: Uuid,
    pub facility_name: String,
    pub facility_type: String,
    pub currency_id: Uuid,
    pub approved_limit: BigDecimal,
    pub interest_rate: Option<BigDecimal>,
    pub interest_type: Option<String>,
    pub effective_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    #[serde(default)]
    pub collateral_required: bool,
}

// ============================================================
// DEBT DRAWDOWN
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DebtDrawdown {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub drawdown_date: NaiveDate,
    pub amount: BigDecimal,
    pub reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================
// CREATE DRAWDOWN
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateDrawdown {
    pub facility_id: Uuid,
    pub drawdown_date: NaiveDate,
    pub amount: BigDecimal,
    pub reference: Option<String>,
}

// ============================================================
// DEBT REPAYMENT
// ============================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DebtRepayment {
    pub id: Uuid,
    pub facility_id: Uuid,
    pub repayment_date: NaiveDate,
    pub principal_amount: BigDecimal,
    pub interest_amount: BigDecimal,
    pub journal_entry_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================
// CREATE REPAYMENT
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateRepayment {
    pub facility_id: Uuid,
    pub repayment_date: NaiveDate,
    pub principal_amount: BigDecimal,
    #[serde(default)]
    pub interest_amount: BigDecimal,
}
