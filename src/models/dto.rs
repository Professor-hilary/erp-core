use chrono::NaiveDate;
// src/models/dto.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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
pub struct TenantRow {
    pub company_id: Uuid,
    pub db_name: String,
    pub db_host: String,
    pub db_port: i32,
    pub db_user: String,
    pub db_password: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct FinancialPeriodDto {
    pub uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_locked: bool,
}

#[derive(sqlx::FromRow)]
pub struct PeriodRow {
    pub uuid: Uuid,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub is_locked: Option<bool>,
}
