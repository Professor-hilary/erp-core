// Accounts models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;


#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[sqlx(rename = "type")]
    pub type_: String,  // e.g., 'asset', 'liability', etc.
    pub balance: f64,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub type_: String,
}
