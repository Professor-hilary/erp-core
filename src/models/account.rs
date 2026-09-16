// src/models/account.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::NaiveDate;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: Uuid,                         // Required, Auto_gen
    pub serial_id: i64,                     // Required, Auto_gen
    pub name: String,                       // Required
    pub code: String,                       // Required
    pub parent_code: Option<String>,        // Optional, Null Default
    pub normal_balance: String,             // Required
    pub current_balance: BigDecimal,        // Required
    pub is_contra: bool,                    // Optional, False Default
    pub is_active: bool,                    // Optional, False Default
    pub created_at: DateTime<Utc>,          // Required, Auto_gen
    pub updated_at: DateTime<Utc>,          // Required, Auto_gen
    pub category: String,                   // Required
    pub cash_flow_category: Option<String>, // Optional, Null Default
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CreateAccount {
    pub name: String,
    pub category: String,
    pub code: String,
    pub normal_balance: String,
    pub parent_code: String,
    pub is_contra: bool,
    pub cash_flow_category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChartOfAccountsEntry {
    pub code: String,
    pub name: String,
    pub category: String,
    pub parent_code: Option<String>,
    pub cash_flow_category: Option<String>,
    pub normal_balance: String,
    pub is_contra: bool,
}

#[derive(serde::Deserialize)]
pub struct CoaTemplate {
    // We must declare them so deserialization works
    pub industry: String,
    pub name: String,
    pub accounts: Vec<ChartOfAccountsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Currency {
    pub id: Uuid,
    pub serial_id: i64,
    pub code: String, // ISO 4217, e.g. "UGX", "USD"
    pub name: String,
    pub decimal_places: i16,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCurrency {
    #[validate(length(equal = 3))]
    pub code: String,
    pub name: String,
    pub decimal_places: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCurrency {
    #[validate(length(equal = 3))]
    pub code: Option<String>,
    pub name: Option<String>,
    pub decimal_places: Option<i16>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExchangeRate {
    pub id: Uuid,
    pub serial_id: i64,
    pub from_currency_id: Uuid,
    pub to_currency_id: Uuid,
    pub rate_date: NaiveDate,
    pub rate: BigDecimal,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpsertExchangeRate {
    pub from_currency_id: Uuid,
    pub to_currency_id: Uuid,
    pub rate_date: NaiveDate,
    pub rate: BigDecimal,
    pub source: Option<String>,
}
