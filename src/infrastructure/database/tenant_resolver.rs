// src/infrastructure/database/tenant_resolver.rs
use sqlx::PgPool;
use uuid::Uuid;

use crate::{models::dto::TenantRow, state::AppState};

pub async fn get_tenant_pool(state: &AppState, company_id: Uuid) -> Result<PgPool, sqlx::Error> {
    // 1. Check cache for fully setup user
    if let Some(pool) = state.tenant_pools.get(&company_id) {
        return Ok(pool.value().clone());
    }

    // 2. Load tenant config from master DB
    let tenant: TenantRow = sqlx::query_as("SELECT * FROM tenant_secrets WHERE company_id = $1")
        .bind(company_id)
        .fetch_one(&state.master_pool)
        .await?;

    // 3. Build connection string
    let url: String = format!(
        "postgres://{}:{}@{}:{}/{}",
        tenant.db_user, tenant.db_password, tenant.db_host, tenant.db_port, tenant.db_name,
    );

    // 4. Create pool
    let pool = PgPool::connect(&url).await?;

    // 5. Cache pool
    state.tenant_pools.insert(company_id, pool.clone());

    Ok(pool)
}
