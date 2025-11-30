// src/features/auth/repository.rs
use crate::infrastructure::errors::AppError;
use crate::models::user::User;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, email: &str, password_hash: &str) -> Result<User, AppError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    async fn get_user_company(
        &self,
        user_id: Uuid,
    ) -> Result<(Option<Uuid>, Option<String>), AppError>;

    async fn get_company_for_switch(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<String>, AppError>;
}

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
    /// # Create Company
    /// Create a new user and password for company operations.
    async fn create(&self, email: &str, password_hash: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING *",
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db) if db.constraint() == Some("users_email_key") => {
                AppError::BadRequest("Email already taken".into())
            }
            _ => AppError::Database(e),
        })?;
        Ok(user)
    }

    /// # Find User
    /// This function finds user using provided email.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    /// # Get Company
    /// This function returns company for a particular company.
    async fn get_user_company(
        &self,
        user_id: Uuid,
    ) -> Result<(Option<Uuid>, Option<String>), AppError> {
        let row = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            SELECT c.uuid, c.tenant_db_name
            FROM companies c
            JOIN user_companies uc ON c.uuid = uc.company_id
            WHERE uc.user_id = $1 AND c.status != 'deleted'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .map(|(id, db)| (Some(id), Some(db)))
            .unwrap_or((None, None)))
    }

    /// # Get Company
    /// This function will return an error if database script fails or server error occurs.
    async fn get_company_for_switch(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let row = sqlx::query_as::<_, (String,)>(
            // returns (tenant_db_name,)
            r#"
            SELECT c.tenant_db_name
            FROM companies c
            JOIN user_companies uc ON c.uuid = uc.company_id
            WHERE uc.user_id = $1 AND c.uuid = $2 AND c.status != 'deleted'
            "#,
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(db_name,)| db_name))
    }
}
