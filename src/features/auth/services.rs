// src/features/auth/services.rs
use crate::infrastructure::errors::AppError;
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

    /// Sign up user into master database, update tenant pools in state
    pub async fn register(&self, user: &CreateUser) -> Result<(User, String), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::BadRequest("Email and password required".into()));
        }

        let password_hash: String = hash(&user.password, DEFAULT_COST)
            .map_err(|_| AppError::Internal("Failed to hash password".into()))?;

        let created_user: User = self.repo.create(&user.email, &password_hash).await?;

        let (company_id, tenant_db) = self.repo.get_user_company(created_user.uuid).await?;

        let token: String = self.generate_token(created_user.uuid, company_id, tenant_db)?;

        Ok((created_user, token))
    }

    /// Sign in user from master database, update tenant pools in state
    pub async fn login(&self, user: &LoginUser) -> Result<String, AppError> {
        let db_user: User = self
            .repo
            .find_by_email(&user.email)
            .await?
            .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

        if !verify(&user.password, &db_user.password_hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?
        {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        let (company_id, tenant_db) = self.repo.get_user_company(db_user.uuid).await?;

        self.generate_token(db_user.uuid, company_id, tenant_db)
    }

    /// Switch company for current user
    pub async fn switch_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<String, AppError> {
        let tenant_db: Option<String> = self
            .repo
            .get_company_for_switch(user_id, company_id)
            .await?;

        if tenant_db.is_none() {
            return Err(AppError::Unauthorized("Not a member of this company".into()));
        }

        self.generate_token(user_id, Some(company_id), tenant_db)
    }

    /// Pure function for generating login and signup tokens
    pub fn generate_token(
        &self,
        user_id: Uuid,
        company_id: Option<Uuid>,
        tenant_db: Option<String>,
    ) -> Result<String, AppError> {
        let claims: JwtClaims = JwtClaims {
            sub: user_id,
            company_id,
            tenant_db,
            exp: (Utc::now() + Duration::weeks(24)).timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Internal("Failed to generate JWT".into()))
    }
}
