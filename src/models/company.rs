// src/models/company.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Company {
    pub uuid: Uuid,
    pub name: String,
    pub slug: String,
    pub tenant_db_name: String,
    pub tenant_db_uri: String,
    pub industry: String,
    pub business_type: String,
    pub currency: String,
    pub country: String,
    pub company_email: Option<String>,
    pub legal_name: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub co_address: Option<String>,
    pub city: Option<String>,
    pub co_state: Option<String>,
    pub zip_code: Option<String>,
    pub tax_id: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_type: Option<String>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyDto {
    pub name: String,
    pub industry: String,
    pub country: String,
    pub currency: String,
    pub company_email: Option<String>,
    pub legal_name: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub co_address: Option<String>,
    pub city: Option<String>,
    pub co_state: Option<String>,
    pub zip_code: Option<String>,
    pub tax_id: Option<String>,
    pub business_type: String,
    pub period_start: Option<NaiveDate>,
    pub period_type: Option<PeriodType>,
    pub custom_end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyDto {
    pub name: Option<String>,
    pub industry: Option<String>,
    pub business_type: Option<String>,
    pub country: Option<String>,
    pub company_email: Option<String>,
    pub legal_name: Option<String>,
    pub currency: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub co_address: Option<String>,
    pub city: Option<String>,
    pub co_state: Option<String>,
    pub zip_code: Option<String>,
    pub tax_id: Option<String>,
    pub period_start: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum PeriodType {
    Monthly,
    Quarterly,
    HalfYearly,
    Yearly,
    Custom,
}

#[derive(Debug, Deserialize)]
pub struct CreateInitialPeriod {
    pub period_type: PeriodType,
    pub start_date: NaiveDate,
    pub custom_end_date: Option<NaiveDate>,
}
