// src/features/auth/services.rs
use crate::errors::AppError;
use crate::features::auth::repository::UserRepository;
use crate::models::dto::JwtClaims;
use crate::models::user::{CreateUser, LoginUser, User};
use crate::state::AppState;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService<R: UserRepository> {
    repo: R,
    state: Arc<AppState>,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repo: R, state: Arc<AppState>) -> Self {
        Self { repo, state }
    }

    // Helper: get user's companies and auto-select one if possible
    async fn get_user_company(
        &self,
        user_id: Uuid,
    ) -> Result<(Option<Uuid>, Option<String>), AppError> {
        let row: Option<(Uuid, String)> = sqlx::query_as(
            r#"
            SELECT c.uuid, c.tenant_db_name
            FROM companies c
            JOIN user_companies uc ON c.uuid = uc.company_id
            WHERE uc.user_id = $1 AND c.status != 'deleted'
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.state.master_pool)
        .await?;

        Ok(row
            .map(|(id, db)| (Some(id), Some(db)))
            .unwrap_or((None, None)))
    }

    pub async fn register(&self, user: &CreateUser) -> Result<(User, String), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::Validation("Email and password required".into()));
        }

        let password_hash = hash(&user.password, DEFAULT_COST)
            .map_err(|_| AppError::Validation("Failed to hash password".into()))?;

        let created_user = self
            .repo
            .create(
                /* &self.state.master_pool, */ &user.email,
                &password_hash,
            )
            .await?;

        let (company_id, tenant_db) = self.get_user_company(created_user.id).await?;

        let token = self.generate_token(created_user.id, company_id, tenant_db)?;

        Ok((created_user, token))
    }

    pub async fn login(&self, user: &LoginUser) -> Result<String, AppError> {
        let db_user = self
            .repo
            .find_by_email(/* &self.state.master_pool, */ &user.email)
            .await?
            .ok_or(AppError::Auth("Invalid credentials".into()))?;

        if !verify(&user.password, &db_user.password_hash)
            .map_err(|_| AppError::Auth("Invalid credentials".into()))?
        {
            return Err(AppError::Auth("Invalid credentials".into()));
        }

        let (company_id, tenant_db) = self.get_user_company(db_user.id).await?;

        self.generate_token(db_user.id, company_id, tenant_db)
    }

    // Shared token generation logic
    fn generate_token(
        &self,
        user_id: Uuid,
        company_id: Option<Uuid>,
        tenant_db: Option<String>,
    ) -> Result<String, AppError> {
        let claims = JwtClaims {
            sub: user_id,
            company_id,
            tenant_db,
            exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Internal("Failed to generate token".into()))
    }

    // New: Switch company
    pub async fn switch_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<String, AppError> {
        // Verify membership
        let company: Option<(String,)> = sqlx::query_as(
            "SELECT c.tenant_db_name
         FROM companies c
         JOIN user_companies uc ON c.uuid = uc.company_id
         WHERE uc.user_id = $1 AND c.uuid = $2",
        )
        .bind(user_id)
        .bind(company_id)
        .fetch_optional(&self.state.master_pool)
        .await?;

        let tenant_db = company.map(|(db_name,)| db_name);

        // RETURN the token
        let token = self.generate_token(user_id, Some(company_id), tenant_db)?;

        Ok(token)
    }
}
