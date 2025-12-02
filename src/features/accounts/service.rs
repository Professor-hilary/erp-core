use std::sync::Arc;

// src/features/accounts/service.rs
use crate::features::accounts::repository::AccountRepository;
use crate::infrastructure::errors::AppError;
use crate::models::account::{Account, CreateAccount};
use crate::state::AppState;

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
        let valid_types: [&str; 5] = ["Asset", "Liability", "Equity", "Revenue", "Expense"];
        if !valid_types.contains(&acc.type_.as_str()) {
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
        println!("JWT Scrt: {}", state.jwt_secret);
        println!("Base Url: {}", state.tenant_config.base_url);
        println!("COA Path: {}", state.coa_seed_path);
        // println!("MasterID: {}", state.master_pool);

        for entry in state.tenant_pools.iter(){
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
            .ok_or(AppError::NotFound("Account not found".into()))
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
        if !valid_types.contains(&updates.type_.as_str()) {
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
        // .ok_or(AppError::NotFound("Account not found".into()))

        // if result.rows_affected() == 0 {
        //     return Err(AppError::NotFound("Account not found".into()));
        // }
        // Ok(())
    }
}
