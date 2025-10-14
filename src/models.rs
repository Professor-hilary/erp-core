// models.rs
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub description: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: f64,
    pub date: NaiveDate,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// DTOs for requests/responses (separation from DB models)
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransaction {
    pub description: Option<String>,
    pub debit_account_id: Uuid,
    pub credit_account_id: Uuid,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct TrialBalance {
    pub account_id: Uuid,
    pub name: String,
    pub balance: f64,
}
