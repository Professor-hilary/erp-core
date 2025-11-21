// src/features/auth/repository.rs
use crate::errors::AppError;
use crate::models::user::User;
use async_trait::async_trait;
use sqlx::PgPool;

// Trait for abstraction (injectable for testing)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        // pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError>;
    async fn find_by_email(&self, /* pool: &PgPool, */ email: &str) -> Result<Option<User>, AppError>;
}

// Concrete impls
#[allow(dead_code)]
pub struct PostgresUserRepo {
    pool: PgPool,
}

impl PostgresUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepo {
    async fn create(
        &self,
        // pool: &PgPool,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        let user: User = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }

    async fn find_by_email(&self/* , pool: &PgPool */, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
}
