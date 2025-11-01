// repositories.rs
use crate::models::account::{Account, CreateAccount};
use crate::errors::AppError;
use sqlx::PgPool;
use async_trait::async_trait;

// Trait for abstraction (injectable for testing)
#[async_trait]
pub trait AccountRepository: Send + Sync {
    fn pool(&self)->&PgPool;
    async fn create(&self, user_id: sqlx::types::Uuid, acc: &CreateAccount) -> Result<Account, AppError>;
    async fn find_by_id(&self, id: sqlx::types::Uuid, user_id: sqlx::types::Uuid) -> Result<Option<Account>, AppError>;
    async fn find_by_user(&self, user_id: sqlx::types::Uuid) -> Result<Vec<Account>, AppError>;
    async fn update_balance(&self, id: sqlx::types::Uuid, delta: f64) -> Result<(), AppError>;
    // async fn list_all(&self, id: sqlx::types::Uuid) -> Result<(), AppError>;
}

// Concrete impls
pub struct PostgresAccountRepo {
    pool: PgPool,
}

impl PostgresAccountRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepo {
    /// # Purpose
    /// Create a new user and insert into the database. User may be employee, admin, or owner.
    /// Purpose of user is authentication and authorization to access and use the application.
    /// User will be used to perform sign in/signup, DBMS CRUD functions and any relevant jobs.
    ///
    /// # Errors
    ///
    /// This function will return an error if user cannot be created.
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

    fn pool(&self)->&PgPool{
        &self.pool
    }
}
