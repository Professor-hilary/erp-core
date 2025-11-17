// src/models/company.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, FromRow)]
pub struct Company {
    pub uuid: Uuid,
    pub name: String,
    pub slug: String,
    pub tenant_db_name: String,
    pub tenant_db_uri: String,
    pub industry: String,
    pub business_type: String,
    pub status: String,
    pub created_by: Uuid,

    // THIS IS THE KEY: column is called `created_at` in DB
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompanyDto {
    pub name: String,
    pub industry: String,
    pub business_type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyDto {
    pub name: Option<String>,
    pub industry: Option<String>,
    pub business_type: Option<String>,
}