use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub uuid: Uuid,
    pub email: String,
    pub password_hash: String,
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

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct UserCompany {
    pub uuid: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub name: Option<String>,
    pub tenant_db_name: Option<String>,
    pub industry: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<String>,
    pub role: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}