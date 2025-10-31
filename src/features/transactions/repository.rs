// src/features/transactions/repositories.rs
use crate::features::accounts::repository::{AccountRepository, PostgresAccountRepo};
use crate::models::transaction::CreateTransaction;
use crate::models::{ Transaction};
use crate::errors::AppError;
use sqlx::PgPool;
use async_trait::async_trait;

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create(&self, user_id: sqlx::types::Uuid, tx: &CreateTransaction) -> Result<Transaction, AppError>;
    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Transaction>, AppError>;
}

// Concrete impls
#[allow(dead_code)]
pub struct PostgresUserRepo {
    pool: PgPool,
}

// Concrete impls
pub struct PostgresTransactionRepo {
    pool: PgPool,
}

impl PostgresTransactionRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepo {
    async fn create(&self, user_id: sqlx::types::Uuid, tx: &CreateTransaction) -> Result<Transaction, AppError> {
        // Enforce business rule: Debit and credit accounts must differ
        if tx.debit_account_id == tx.credit_account_id {
            return Err(AppError::Validation("Debit and credit accounts must differ".to_string()));
        }

        let transaction = sqlx::query_as::<_, Transaction>(
            "INSERT INTO transactions (description, debit_account_id, credit_account_id, amount, user_id) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(&tx.description)
        .bind(tx.debit_account_id)
        .bind(tx.credit_account_id)
        .bind(tx.amount)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Update balances (double-entry: debit increases, credit decreases)
        let debit_repo = PostgresAccountRepo::new(self.pool.clone());
        let credit_repo = PostgresAccountRepo::new(self.pool.clone());
        debit_repo.update_balance(tx.debit_account_id, tx.amount).await?;
        credit_repo.update_balance(tx.credit_account_id, -tx.amount).await?;

        Ok(transaction)
    }

    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Transaction>, AppError> {
        let transactions = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE user_id = $1 ORDER BY date DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(transactions)
    }
}
