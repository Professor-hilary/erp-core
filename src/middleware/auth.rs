use chrono::NaiveDate;
use uuid::Uuid;

// Always available if JWT is valid
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

// Only available if company is selected AND tenant DB is cached
#[derive(Debug, Clone)]
pub struct AuthenticatedTenant {
    pub user_id: Uuid,                   // Every operation requires a valid user
    pub company_id: Uuid,                // Company for a valid in tenant account
    pub tenant_pool: sqlx::PgPool,       // Pool for performing sqlx transactions
    pub current_period_start: NaiveDate, // Financial period start: month, yr,...
    pub current_period_end: NaiveDate,   // Financial period close
    pub period_is_locked: bool,          // Whether to post outside financial period
}
