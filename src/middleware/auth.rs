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
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub tenant_pool: sqlx::PgPool,
    pub current_period_start: NaiveDate,
    pub current_period_end: NaiveDate,
    pub period_is_locked: bool,
}
