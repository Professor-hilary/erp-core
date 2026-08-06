// src/models/account.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::fixed_assets;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetClass {
    pub class_id: Uuid,
    pub user_id: Uuid,
    pub class_name: String,
    pub category_type: String, // PPE, INTANGIBLE, etc.
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdditionalCost {
    pub description: String,
    pub amount: BigDecimal,
}

#[derive(Debug, Clone)]
pub struct CapitalizationResult {
    pub asset_id: Uuid,
    pub capitalized_amount: BigDecimal,
    pub breakdown: Option<serde_json::Value>,
    pub additional_costs_count: usize,
}


#[derive(Debug, Deserialize)]
pub struct CreateAssetClass {
    pub class_name: String,
    pub category_type: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CapitalizeAssetOrCWIP {
    pub asset_uuid: Uuid,
    pub date: Option<NaiveDate>,
    pub capitalized_amount: Option<BigDecimal>,
    pub costs: Option<Vec<fixed_assets::AdditionalCost>>,
    // pub costs: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FixedAsset {
    pub asset_id: Uuid,
    pub user_id: Uuid,
    pub asset_code: String,
    pub asset_name: String,
    pub description: Option<String>,

    pub class_id: Option<Uuid>,
    pub location: Option<String>,
    pub department: Option<String>,
    pub custodian_id: Option<Uuid>,

    pub acquisition_date: NaiveDate,
    pub supplier_id: Option<Uuid>,
    pub po_reference: Option<String>,

    pub original_cost: BigDecimal, // Better for money
    pub capitalized_amount: Option<BigDecimal>,
    pub is_capitalized: bool,
    pub capitalization_date: Option<NaiveDate>,

    pub financing_method: Option<String>,
    pub tax_class: Option<String>,

    pub useful_life_years: i32,
    pub residual_value: BigDecimal,
    pub depreciation_method: String,
    pub depreciation_rate: Option<BigDecimal>,
    pub depreciation_start_date: Option<NaiveDate>,

    pub status: String,

    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAsset {
    pub asset_code: String,
    pub asset_name: String,
    pub description: Option<String>,
    pub class_id: Option<Uuid>,
    pub location: Option<String>,
    pub department: Option<String>,
    pub custodian_id: Option<Uuid>,
    pub acquisition_date: NaiveDate,
    pub supplier_id: Option<Uuid>,
    pub tax_class: Option<String>,
    pub po_reference: Option<String>,
    pub original_cost: BigDecimal,
    pub capitalized_amount: Option<BigDecimal>,
    pub is_capitalized: bool,
    pub capitalization_date: Option<NaiveDate>,
    pub financing_method: Option<String>,
    pub useful_life_years: i32,
    pub residual_value: BigDecimal,
    pub depreciation_method: String,
    pub depreciation_rate: Option<BigDecimal>,
    pub depreciation_start_date: Option<NaiveDate>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetBook {
    pub book_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub book_type: String, // FINANCIAL, TAX, IFRS, etc.
    pub useful_life_years: Option<i32>,
    pub depreciation_method: Option<String>,
    pub depreciation_rate: Option<BigDecimal>,
    pub residual_value: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetBook {
    pub asset_id: Uuid,
    pub book_type: String,
    pub useful_life_years: Option<i32>,
    pub depreciation_method: Option<String>,
    pub depreciation_rate: Option<BigDecimal>,
    pub residual_value: Option<BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CapitalWorkInProgress {
    pub cwip_id: Uuid,
    pub user_id: Uuid,
    pub project_name: String,
    pub total_accumulated_cost: BigDecimal,
    pub start_date: NaiveDate,
    pub expected_completion_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCWIP {
    pub project_name: String,
    pub total_accumulated_cost: BigDecimal,
    pub start_date: NaiveDate,
    pub expected_completion_date: Option<NaiveDate>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetDepreciation {
    pub dep_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub book_id: Option<Uuid>,
    pub period_date: NaiveDate,
    pub depreciation_amount: BigDecimal,
    pub accumulated_depreciation: BigDecimal,
    pub nbv: BigDecimal,
    pub posted_to_gl: bool,
    pub created_at: DateTime<Utc>,
    pub financial_depreciation: BigDecimal,
    pub accumulated_tax_depreciation: BigDecimal,
    pub nbv_financial: BigDecimal,
    pub twdv: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetComponent {
    pub component_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub component_name: String,
    pub cost: BigDecimal,
    pub useful_life_years: Option<i32>,
    pub depreciation_start_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetComponent {
    pub asset_id: Uuid,
    pub component_name: String,
    pub cost: BigDecimal,
    pub useful_life_years: Option<i32>,
    pub depreciation_start_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetMaintenance {
    pub maintenance_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub maintenance_type: Option<String>,
    pub description: Option<String>,
    pub cost: Option<BigDecimal>,
    pub maintenance_date: NaiveDate,
    pub next_due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetMaintenance {
    pub asset_id: Uuid,
    pub maintenance_type: Option<String>,
    pub description: Option<String>,
    pub cost: Option<BigDecimal>,
    pub maintenance_date: NaiveDate,
    pub next_due_date: Option<NaiveDate>,
}

// Insurance
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetInsurance {
    pub insurance_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub policy_number: Option<String>,
    pub insurer: Option<String>,
    pub insured_amount: Option<BigDecimal>,
    pub start_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub premium: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetInsurance {
    pub asset_id: Uuid,
    pub policy_number: Option<String>,
    pub insurer: Option<String>,
    pub insured_amount: Option<BigDecimal>,
    pub start_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub premium: Option<BigDecimal>,
}

// Transactions (Transfer, Revaluation, Disposal)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetTransaction {
    pub trans_id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub transaction_type: String, // Transfer, Revaluation, Disposal, Impairment
    pub transaction_date: NaiveDate,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub amount: Option<BigDecimal>,
    pub proceeds: Option<BigDecimal>,
    pub gain_loss: Option<BigDecimal>,
    pub reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

pub type AssetTransfer = AssetTransaction;
pub type AssetRevaluation = AssetTransaction;
pub type AssetDisposal = AssetTransaction;

#[derive(Debug, Deserialize)]
pub struct CreateAssetTransfer {
    pub asset_id: Uuid,
    pub transfer_date: NaiveDate,
    pub from_location: String,
    pub to_location: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetDisposal {
    pub asset_id: Uuid,
    pub disposal_date: NaiveDate,
    pub proceeds: BigDecimal,
    // pub gain_loss: Option<BigDecimal>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetRevaluation {
    pub asset_id: Uuid,
    pub revaluation_date: NaiveDate,
    pub amount: BigDecimal,
    pub reason: Option<String>,
}
