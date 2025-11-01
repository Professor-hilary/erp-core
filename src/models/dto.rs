// src/models/dto.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
}
