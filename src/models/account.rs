// src/models/account.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub name: String,
    pub code: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub parent_uuid: Option<Uuid>,
    pub normal_balance: String,
    pub is_contra: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub subtype: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub type_: String,
}
