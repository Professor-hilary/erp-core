use crate::features::accounts::repository::AccountRepository;
use crate::features::transactions::repository::TransactionRepository;
// services.rs
use crate::models::transaction::{CreateTransaction, Transaction};
use crate::errors::AppError;
// use bcrypt::{hash, verify, DEFAULT_COST};
// use chrono::{Duration, Utc};
// use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
#[allow(dead_code)]
pub struct TransactionService<R: AccountRepository, T: TransactionRepository> {
    account_repo: R,
    tx_repo: T,
}

impl<R: AccountRepository, T: TransactionRepository> TransactionService<R, T> {
    pub fn new(account_repo: R, tx_repo: T) -> Self { Self { account_repo, tx_repo } }

    pub async fn create_transaction(&self, user_id: Uuid, tx: &CreateTransaction) -> Result<Transaction, AppError> {
        // Business rule: Ensure accounts exist and belong to user
        let debit_acc = self.account_repo.find_by_id(tx.debit_account_id, user_id).await?
            .ok_or(AppError::NotFound)?;
        let credit_acc = self.account_repo.find_by_id(tx.credit_account_id, user_id).await?
            .ok_or(AppError::NotFound)?;
        if debit_acc.user_id != user_id || credit_acc.user_id != user_id {
            return Err(AppError::Validation("Accounts must belong to user".to_string()));
        }
        self.tx_repo.create(user_id, tx).await
    }

    pub async fn get_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, AppError> {
        self.tx_repo.find_by_user(user_id).await
    }
}
