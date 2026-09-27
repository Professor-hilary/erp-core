// src/state.rs

use chrono::NaiveDate;
use dashmap::DashMap;
use moka::future::Cache;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::infrastructure::redis_blacklist::SharedBlacklist;

#[derive(Clone, Debug)]
pub struct JwtConfig {
    pub access_expiry_secs: i64,  // default 900
    pub refresh_expiry_secs: i64, // default 60 days
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            access_expiry_secs: 900,
            refresh_expiry_secs: 60 * 24 * 3600,
        }
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct AppState {
    pub master_pool: PgPool,
    pub tenant_pools: DashMap<Uuid, PgPool>,
    pub jwt_secret: String,
    pub jwt_config: JwtConfig,
    pub coa_seed_path: String,
    pub tenant_config: TenantConfig,
    pub period_cache: Cache<Uuid, PeriodInfo>,
    pub token_blacklist: Option<SharedBlacklist>,
}

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

// #[derive(Clone, Debug, FromRow)]
// pub struct AppState {
//     pub master_pool: PgPool,
//     pub tenant_pools: DashMap<Uuid, PgPool>,
//     pub jwt_secret: String,
//     pub coa_seed_path: String,
//     pub tenant_config: TenantConfig,
//     pub period_cache: Cache<Uuid, PeriodInfo>,
// }
