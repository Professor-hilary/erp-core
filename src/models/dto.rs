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

#[derive(sqlx::FromRow)]
pub struct TenantRow{
    pub _company_id: Uuid,
    pub db_name: String,
    pub db_host: String,
    pub db_port: i32,
    pub db_user: String,
    pub db_password: String,
}