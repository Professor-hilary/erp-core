use chrono::NaiveDate;
// src/models/dto.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub company_id: Option<Uuid>,
    pub tenant_db: Option<String>,
    pub exp: usize,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
pub struct TenantRow{
    pub company_id: Uuid,
    pub db_name: String,
    pub db_host: String,
    pub db_port: i32,
    pub db_user: String,
    pub db_password: String,

    // Financial period
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
}