// src/features/transactions/service.rs
use crate::features::accounts::repository::AccountRepository;
use crate::features::transactions::repository::TransactionRepository;
use crate::infrastructure::errors::AppError;
use crate::models::transaction::{CreateTransaction, Transaction};
use bigdecimal::{BigDecimal, Zero};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TransactionService<R: AccountRepository, T: TransactionRepository> {
    account_repo: R,
    tx_repo: T,
}

impl<R: AccountRepository, T: TransactionRepository> TransactionService<R, T> {
    pub fn new(account_repo: R, tx_repo: T) -> Self {
        Self {
            account_repo,
            tx_repo,
        }
    }

    pub async fn create_transaction(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        // Validate accounts exist and belong to user
        let _debit_acc = self
            .account_repo
            .find_by_uuid(tenant_pool, tx.debit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Debit account not found".into()))?;
        let _credit_acc = self
            .account_repo
            .find_by_uuid(tenant_pool, tx.credit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Credit account not found".into()))?;

        // if debit_acc.user_id != user_id || credit_acc.user_id != user_id {
        //     return Err(AppError::Validation("Accounts must belong to user".into()));
        // }

        match tx.amount <= BigDecimal::zero() {
            true => return Err(AppError::BadRequest("Amount must be positive".into())),
            false => (),
        }

        // Create transaction
        let transaction = self.tx_repo.create(tenant_pool, user_id, tx).await?;

        // Update account balances
        self.account_repo
            .update_balance(tenant_pool, tx.debit_account_id, -tx.amount.clone())
            .await?;
        self.account_repo
            .update_balance(tenant_pool, tx.credit_account_id, tx.amount.clone())
            .await?;

        Ok(transaction)
    }

    pub async fn get_transaction(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Transaction, AppError> {
        self.tx_repo
            .find_by_id(tenant_pool, id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))
    }

    pub async fn get_transactions(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, AppError> {
        self.tx_repo.find_by_user(tenant_pool, user_id).await
    }

    pub async fn update_transaction(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        // Get old transaction to reverse balances
        let old = self
            .tx_repo
            .find_by_id(tenant_pool, id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))?;

        // Validate new accounts
        let _debit_acc = self
            .account_repo
            .find_by_uuid(tenant_pool, tx.debit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("New debit account not found".into()))?;
        let _credit_acc = self
            .account_repo
            .find_by_uuid(tenant_pool, tx.credit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("New credit account not found".into()))?;

        // Reverse old balances
        self.account_repo
            .update_balance(tenant_pool, old.debit_account_id, old.amount.clone())
            .await?;
        self.account_repo
            .update_balance(tenant_pool, old.credit_account_id, -old.amount.clone())
            .await?;

        // Apply new balances
        self.account_repo
            .update_balance(tenant_pool, tx.debit_account_id, -tx.amount.clone())
            .await?;
        self.account_repo
            .update_balance(tenant_pool, tx.credit_account_id, tx.amount.clone())
            .await?;

        // Update transaction
        self.tx_repo.update(tenant_pool, id, user_id, tx).await
    }

    pub async fn delete_transaction(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let tx = self
            .tx_repo
            .find_by_id(tenant_pool, id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))?;

        // Reverse balances
        self.account_repo
            .update_balance(tenant_pool, tx.debit_account_id, tx.amount.clone())
            .await?;
        self.account_repo
            .update_balance(tenant_pool, tx.credit_account_id, -tx.amount.clone())
            .await?;

        self.tx_repo.delete(tenant_pool, id, user_id).await
    }
}
