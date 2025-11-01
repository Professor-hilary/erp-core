// src/features/transactions/service.rs
use crate::errors::AppError;
use crate::features::accounts::repository::AccountRepository;
use crate::features::transactions::repository::TransactionRepository;
use crate::models::transaction::{CreateTransaction, Transaction};
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
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        // Validate accounts exist and belong to user
        let debit_acc = self
            .account_repo
            .find_by_id(tx.debit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Debit account not found".into()))?;
        let credit_acc = self
            .account_repo
            .find_by_id(tx.credit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("Credit account not found".into()))?;

        if debit_acc.user_id != user_id || credit_acc.user_id != user_id {
            return Err(AppError::Validation("Accounts must belong to user".into()));
        }

        if tx.amount <= 0.0 {
            return Err(AppError::Validation("Amount must be positive".into()));
        }

        // Create transaction
        let transaction = self.tx_repo.create(user_id, tx).await?;

        // Update account balances
        self.account_repo
            .update_balance(tx.debit_account_id, -tx.amount)
            .await?;
        self.account_repo
            .update_balance(tx.credit_account_id, tx.amount)
            .await?;

        Ok(transaction)
    }

    pub async fn get_transaction(&self, id: Uuid, user_id: Uuid) -> Result<Transaction, AppError> {
        self.tx_repo
            .find_by_id(id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))
    }

    pub async fn get_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, AppError> {
        self.tx_repo.find_by_user(user_id).await
    }

    pub async fn update_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        // Get old transaction to reverse balances
        let old = self
            .tx_repo
            .find_by_id(id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))?;

        // Validate new accounts
        let _debit_acc = self
            .account_repo
            .find_by_id(tx.debit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("New debit account not found".into()))?;
        let _credit_acc = self
            .account_repo
            .find_by_id(tx.credit_account_id, user_id)
            .await?
            .ok_or(AppError::NotFound("New credit account not found".into()))?;

        // Reverse old balances
        self.account_repo
            .update_balance(old.debit_account_id, old.amount)
            .await?;
        self.account_repo
            .update_balance(old.credit_account_id, -old.amount)
            .await?;

        // Apply new balances
        self.account_repo
            .update_balance(tx.debit_account_id, -tx.amount)
            .await?;
        self.account_repo
            .update_balance(tx.credit_account_id, tx.amount)
            .await?;

        // Update transaction
        self.tx_repo.update(id, user_id, tx).await
    }

    pub async fn delete_transaction(&self, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let tx = self
            .tx_repo
            .find_by_id(id, user_id)
            .await?
            .ok_or(AppError::NotFound("Transaction not found".into()))?;

        // Reverse balances
        self.account_repo
            .update_balance(tx.debit_account_id, tx.amount)
            .await?;
        self.account_repo
            .update_balance(tx.credit_account_id, -tx.amount)
            .await?;

        self.tx_repo.delete(id, user_id).await
    }
}
