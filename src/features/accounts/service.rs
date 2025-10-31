use crate::features::transactions::repository::TransactionRepository;
// services.rs
use crate::errors::AppError;
use crate::features::accounts::repository::AccountRepository;
use crate::models::account::{Account, CreateAccount};
// use bcrypt::{hash, verify, DEFAULT_COST};
// use chrono::{Duration, Utc};
// use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;

#[allow(dead_code)]
pub struct AccountingService<R: AccountRepository, T: TransactionRepository> {
    account_repo: R,
    tx_repo: T,
}

impl<R: AccountRepository, T: TransactionRepository> AccountingService<R, T> {
    pub fn new(account_repo: R, tx_repo: T) -> Self {
        Self {
            account_repo,
            tx_repo,
        }
    }

    pub async fn create_account(
        &self,
        user_id: Uuid,
        acc: &CreateAccount,
    ) -> Result<Account, AppError> {
        if !["asset", "liability", "equity", "revenue", "expense"].contains(&acc.type_.as_str()) {
            return Err(AppError::Validation("Invalid account type".to_string()));
        }
        self.account_repo.create(user_id, acc).await
    }

    pub async fn get_accounts(&self, user_id: Uuid) -> Result<Vec<Account>, AppError> {
        self.account_repo.find_by_user(user_id).await
    }

    // pub async fn list_accounts (&self, user_id: Uuid) -> Result<Vec<Account>, AppError> {
    //     self.account_repo.list_all(user_id).await
    // }
}
