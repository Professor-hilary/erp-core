use uuid::Uuid;

// Always available if JWT is valid
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

// Only available if company is selected AND tenant DB is cached
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthenticatedTenant {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub tenant_pool: sqlx::PgPool,
}
