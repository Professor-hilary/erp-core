// repositories.rs
use crate::models::{Account, CreateAccount, Transaction, CreateTransaction, User};
use crate::errors::AppError;
use sqlx::PgPool;
use async_trait::async_trait;

// Trait for abstraction (injectable for testing)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
}

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, user_id: sqlx::types::Uuid, acc: &CreateAccount) -> Result<Account, AppError>;
    async fn find_by_id(&self, id: sqlx::types::Uuid, user_id: sqlx::types::Uuid) -> Result<Option<Account>, AppError>;
    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Account>, AppError>;
    async fn update_balance(&self, id: sqlx::types::Uuid, delta: f64) -> Result<(), AppError>;
}

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create(&self, user_id: sqlx::types::Uuid, tx: &CreateTransaction) -> Result<Transaction, AppError>;
    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Transaction>, AppError>;
}

// Concrete impls
pub struct PostgresUserRepo {
    pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl UserRepository for PostgresUserRepo {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *"
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
}

pub struct PostgresAccountRepo {
    pool: PgPool,
}

impl PostgresAccountRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepo {
    async fn create(&self, user_id: sqlx::types::Uuid, acc: &CreateAccount) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "INSERT INTO accounts (name, type, user_id) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(&acc.name)
        .bind(&acc.type_)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(account)
    }

    async fn find_by_id(&self, id: sqlx::types::Uuid, user_id: sqlx::types::Uuid) -> Result<Option<Account>, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "SELECT * FROM accounts WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(account)
    }

    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Account>, AppError> {
        let accounts = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(accounts)
    }

    async fn update_balance(&self, id: sqlx::types::Uuid, delta: f64) -> Result<(), AppError> {
        sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
            .bind(delta)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

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
