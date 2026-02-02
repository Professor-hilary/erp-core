// src/models/company.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyDto {
    pub name: String,
    pub industry: String,
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
    pub business_type: String,
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyDto {
    pub name: Option<String>,
    pub industry: Option<String>,
    pub business_type: Option<String>,
    pub country: Option<String>,
    pub company_email: Option<String>,
    pub legal_name: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub co_address: Option<String>,
    pub city: Option<String>,
    pub co_state: Option<String>,
    pub zip_code: Option<String>,
    pub tax_id: Option<String>,
    pub fiscal_year_start: Option<NaiveDate>,
    pub fiscal_year_end: Option<NaiveDate>,
}
