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
