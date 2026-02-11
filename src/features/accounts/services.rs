// src/features/accounts/service.rs
use std::sync::Arc;

use crate::{
    features::accounts::repository::AccountRepository,
    interface::api::errors::AppError,
    models::{
        account::{Account, CreateAccount},
        dto::FinancialPeriodDto,
    },
    state::AppState,
};

use sqlx::PgPool;
use uuid::Uuid;

pub struct AccountingService<R: AccountRepository> {
    account_repo: R,
}

impl<R: AccountRepository> AccountingService<R> {
    pub fn new(account_repo: R) -> Self {
        Self { account_repo }
    }

    pub async fn create_account(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        acc: &CreateAccount,
    ) -> Result<Account, AppError> {
        let valid_types: [&str; 5] = ["asset", "liability", "equity", "revenue", "expense"];
        if !valid_types.contains(&acc.category.as_str()) {
            return Err(AppError::BadRequest("Invalid account type".into()));
        }
        self.account_repo.create(tenant_pool, user_id, acc).await
    }

    pub async fn get_accounts(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        state: Arc<AppState>,
    ) -> Result<Vec<Account>, AppError> {
        for entry in state.tenant_pools.iter() {
            println!("Tenant UUID: {}", entry.key());
        }

        self.account_repo.find_by_user(tenant_pool, user_id).await
    }

    pub async fn get_account_by_uuid(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Account, AppError> {
        self.account_repo
            .find_by_uuid(tenant_pool, uuid, user_id)
            .await?
            .ok_or(AppError::NotFound("This item does not exist, create it or try searching different a serial number or id!".into()))
    }

    pub async fn get_account_by_serial(
        &self,
        tenant_pool: &PgPool,
        serial_id: i64,
        user_id: Uuid,
    ) -> Result<Account, AppError> {
        self.account_repo
            .find_by_serial_id(tenant_pool, serial_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Account not found".into()))
    }

    pub async fn update_account(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        updates: &CreateAccount,
    ) -> Result<Account, AppError> {
        let valid_types: [&str; 5] = ["asset", "liability", "equity", "revenue", "expense"];
        if !valid_types.contains(&updates.category.as_str()) {
            return Err(AppError::BadRequest("Invalid account type".into()));
        }

        // Now delegate to repo
        self.account_repo
            .update_account_info(tenant_pool, id, user_id, updates)
            .await
    }

    pub async fn delete_account(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.account_repo
            .delete_account(tenant_pool, id, user_id)
            .await
    }

    pub async fn list_all_periods(
        &self,
        pool: &PgPool,
    ) -> Result<Vec<FinancialPeriodDto>, AppError> {
        self.account_repo.list_periods(pool).await
    }

    pub async fn close_financial_period(
        &self,
        state: Arc<AppState>,
        tenant_pool: &PgPool,
        period_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        self.account_repo
            .close_period(state, period_id, tenant_pool, company_id)
            .await
    }
}
