use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub uuid: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub other_name: Option<String>,
    pub telephone: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: Option<String>,
    pub pref_language: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// DTOs for requests/responses (separation from DB models)
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub other_name: Option<String>,
    pub telephone: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: Option<String>,
    pub pref_language: Option<String>,
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
    pub currency: Option<String>,
    pub name: Option<String>,
    pub tenant_db_name: Option<String>,
    pub industry: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<String>,
    pub role: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub slug: Option<String>,
    pub country: Option<String>,
    pub company_email: Option<String>,
    pub legal_name: Option<String>,
    pub telephone: Option<String>,
    pub website: Option<String>,
    pub co_address: Option<String>,
    pub city: Option<String>,
    pub co_state: Option<String>,
    pub zip_code: Option<String>,
    pub tax_id: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_type: Option<String>,
}
