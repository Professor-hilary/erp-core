// src/models/account.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub uuid: Uuid,                // Required, Auto_gen
    pub serial_id: i64,            // Required, Auto_gen
    pub name: String,              // Required
    pub code: String,              // Required
    pub parent_code: Option<String>,       // Optional, Null Default
    pub normal_balance: String,    // Required
    pub is_contra: bool,           // Optional, False Default
    pub is_active: bool,           // Optional, False Default
    pub created_at: DateTime<Utc>, // Required, Auto_gen
    pub updated_at: DateTime<Utc>, // Required, Auto_gen
    #[sqlx(rename = "type")] // Required
    pub type_: String,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct CreateAccount {
    pub name: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub code: String,
    pub normal_balance: String,
    // pub subtype: String,
    pub parent_code: String,
    pub is_contra: bool,
    // pub is_active: String,
}
