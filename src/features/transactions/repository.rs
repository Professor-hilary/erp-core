use crate::infrastructure::errors::AppError;
use crate::models::Transaction;
use crate::models::transaction::CreateTransaction;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError>;
    async fn find_by_id(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Transaction>, AppError>;
    async fn find_by_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, AppError>;
    async fn update(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError>;
    async fn delete(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError>;
}

// Concrete impls
#[allow(dead_code)]
pub struct PostgresUserRepo;

// Concrete impls
pub struct PostgresTransactionRepo;

impl PostgresTransactionRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepo {
    async fn create(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions
            (description, debit_account_id, credit_account_id, amount, date, user_id)
            VALUES ($1, $2, $3, $4, CURRENT_DATE, $5)
            RETURNING *
            "#,
        )
        .bind(&tx.description)
        .bind(tx.debit_account_id)
        .bind(tx.credit_account_id)
        .bind(&tx.amount)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(transaction)
    }

    async fn find_by_id(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Transaction>, AppError> {
        let tx = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        Ok(tx)
    }

    async fn find_by_user(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Transaction>, AppError> {
        let txs = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE user_id = $1 ORDER BY date DESC, created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        Ok(txs)
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        tx: &CreateTransaction,
    ) -> Result<Transaction, AppError> {
        let updated = sqlx::query_as::<_, Transaction>(
            r#"
            UPDATE transactions
            SET description = $1, debit_account_id = $2, credit_account_id = $3, amount = $4
            WHERE id = $5 AND user_id = $6
            RETURNING *
            "#,
        )
        .bind(&tx.description)
        .bind(tx.debit_account_id)
        .bind(tx.credit_account_id)
        .bind(&tx.amount)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        Ok(updated)
    }

    async fn delete(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Transaction not found".into()));
        }
        Ok(())
    }
}
