// src/features/auth/repository.rs
use crate::interface::api::errors::AppError;
use crate::models::user::{User, UserCompany};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(
        &self,
        email: &str,
        password_hash: &str,
        firstname: &str,
        othername: Option<String>,
        telephone: Option<String>,
        avatar_url: Option<String>,
        timezone: Option<String>,
        pref_language: Option<String>,
    ) -> Result<User, AppError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    async fn get_tenant_url(&self, company_id: Option<Uuid>) -> Result<Option<String>, AppError>;

    async fn get_user_company(&self, user_id: Uuid) -> Result<Option<UserCompany>, AppError>;

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
    async fn create(
        &self,
        email: &str,
        password_hash: &str,
        firstname: &str,
        othername: Option<String>,
        telephone: Option<String>,
        avatar_url: Option<String>,
        timezone: Option<String>,
        pref_language: Option<String>,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (
                email,
                password_hash,
                first_name,
                other_name,
                telephone,
                avatar_url,
                timezone,
                pref_language
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        )
        .bind(email)
        .bind(password_hash)
        .bind(firstname)
        .bind(othername)
        .bind(telephone)
        .bind(avatar_url)
        .bind(timezone)
        .bind(pref_language)
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
        let user: Option<User> = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }

    // # Get Url
    async fn get_tenant_url(&self, company_id: Option<Uuid>) -> Result<Option<String>, AppError> {
        // Get Tenant Secret From Secrets Table
        let secret: Option<(serde_json::Value,)> =
            sqlx::query_as("SELECT secret FROM tenant_secrets WHERE company_id = $1")
                .bind(company_id)
                .fetch_optional(&self.pool)
                .await?;

        let tenant_url: Option<String> =
            secret.and_then(|(value,)| value["tenant_url"].as_str().map(|s: &str| s.to_string()));

        Ok(tenant_url)
    }

    /// # Get Company
    /// This function returns company for a particular company.
    async fn get_user_company(&self, user_id: Uuid) -> Result<Option<UserCompany>, AppError> {
        let row: Option<UserCompany> = sqlx::query_as::<_, UserCompany>(
            r#"
            SELECT
                c.uuid              AS uuid,
                c.uuid              AS company_id,
                c.name              AS name,
                c.currency          AS currency,
                c.tenant_db_name    AS tenant_db_name,
                c.industry          AS industry,
                c.business_type     AS business_type,
                c.status            AS status,
                uc.role             AS role,
                c.created_at        AS created_at,
                c.slug              AS slug,
                c.country           AS country,
                c.company_email     AS company_email,
                c.legal_name        AS legal_name,
                c.telephone         AS telephone,
                c.website           AS website,
                c.co_address        AS co_address,
                c.city              AS city,
                c.co_state          AS co_state,
                c.zip_code          AS zip_code,
                c.tax_id            AS tax_id,
                c.period_start      AS period_start,
                c.period_type       AS period_type
            FROM companies c
            JOIN user_companies uc ON c.uuid = uc.company_id
            WHERE uc.user_id = $1 AND c.status != 'deleted'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    /// # Get Company
    /// This function will return an error if database script fails or server error occurs.
    async fn get_company_for_switch(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let row: Option<(String,)> = sqlx::query_as::<_, (String,)>(
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
