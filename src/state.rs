// src/state.rs

use chrono::NaiveDate;
use dashmap::DashMap;
use moka::future::Cache;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TenantConfig {
    pub base_url: String,
}

#[derive(Clone, Debug)]
pub struct PeriodInfo {
    pub _uuid: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub is_locked: bool,
}

#[derive(Clone, Debug, FromRow)]
pub struct AppState {
    pub master_pool: PgPool,
    pub tenant_pools: DashMap<Uuid, PgPool>,
    pub jwt_secret: String,
    pub coa_seed_path: String,
    pub tenant_config: TenantConfig,
    pub period_cache: Cache<Uuid, PeriodInfo>,
}
