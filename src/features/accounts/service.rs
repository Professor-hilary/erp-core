// src/features/accounts/service.rs
use crate::errors::AppError;
use crate::features::accounts::repository::AccountRepository;
use crate::models::account::{Account, CreateAccount};
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
        user_id: Uuid,
        acc: &CreateAccount,
    ) -> Result<Account, AppError> {
        let valid_types = ["asset", "liability", "equity", "revenue", "expense"];
        if !valid_types.contains(&acc.type_.as_str()) {
            return Err(AppError::Validation("Invalid account type".into()));
        }
        self.account_repo.create(user_id, acc).await
    }

    pub async fn get_accounts(&self, user_id: Uuid) -> Result<Vec<Account>, AppError> {
        self.account_repo.find_by_user(user_id).await
    }

    pub async fn get_account_by_uuid(
        &self,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Account, AppError> {
        self.account_repo
            .find_by_uuid(uuid, user_id)
            .await?
            .ok_or(AppError::NotFound("Account not found".into()))
    }

    pub async fn get_account_by_serial(
        &self,
        serial_id: i64,
        user_id: Uuid,
    ) -> Result<Account, AppError> {
        self.account_repo
            .find_by_serial_id(serial_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Account not found".into()))
    }

    pub async fn update_account(
        &self,
        id: Uuid,
        user_id: Uuid,
        updates: &CreateAccount,
    ) -> Result<Account, AppError> {
        let valid_types = ["asset", "liability", "equity", "revenue", "expense"];
        if !valid_types.contains(&updates.type_.as_str()) {
            return Err(AppError::Validation("Invalid account type".into()));
        }

        // Now delegate to repo
        self.account_repo
            .update_account_info(id, user_id, updates)
            .await
    }

    pub async fn delete_account(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM accounting.accounts WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(self.account_repo.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Account not found".into()));
        }
        Ok(())
    }
}
