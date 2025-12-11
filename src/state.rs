// src/state.rs
use dashmap::DashMap;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct TenantConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub master_pool: PgPool,
    pub tenant_pools: DashMap<Uuid, PgPool>,
    pub jwt_secret: String,
    pub coa_seed_path: String,
    pub tenant_config: TenantConfig,
}
