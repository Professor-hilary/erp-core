// repositories.rs
use crate::models::user::{User};
use crate::errors::AppError;
use sqlx::PgPool;
use async_trait::async_trait;

// Trait for abstraction (injectable for testing)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
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

// pub struct PostgresAccountRepo {
//     pool: PgPool,
// }

