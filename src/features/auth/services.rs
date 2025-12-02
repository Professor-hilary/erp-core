// src/features/auth/services.rs
use crate::features::auth::repository::UserRepository;
use crate::infrastructure::errors::AppError;
use crate::models::dto::JwtClaims;
use crate::models::user::{CreateUser, LoginUser, User};
use crate::state::AppState;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::postgres::PgPoolOptions;
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
    pub async fn register(
        &self,
        user: &CreateUser,
        _state: Arc<AppState>,
    ) -> Result<(User, String), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::BadRequest("Email and password required".into()));
        }

        let password_hash: String = hash(&user.password, DEFAULT_COST)
            .map_err(|_| AppError::Internal("Failed to hash password".into()))?;

        let created_user: User = self.repo.create(&user.email, &password_hash).await?;

        let (company_id, tenant_db) = self.repo.get_user_company(created_user.uuid).await?;

        let token: String =
            self.generate_token(created_user.uuid, company_id, tenant_db.clone())?;

        Ok((created_user, token))
    }

    /// Sign in user from master database, update tenant pools in state
    pub async fn login(&self, user: &LoginUser, _state: Arc<AppState>) -> Result<(User, String), AppError> {
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

        let token = self.generate_token(db_user.uuid, company_id, tenant_db.clone())?;

        // Get Tenant Secret From Secrets Table
        let tenant_url_option: Option<String> = self.repo.get_tenant_url(company_id).await?;
        let tenant_url: &str = tenant_url_option
            .as_deref()
            .ok_or(AppError::Internal("Tenant Url missing".to_string()))?;

        // Lazy Load ensures company database is loaded
        let company_uuid: Uuid =
            company_id.ok_or_else(|| AppError::Internal("Company ID missing".into()))?;

        Self::ensure_tenant_pool(&_state, company_uuid, tenant_url.as_ref()).await?;

        Ok((db_user, token))
    }

    /// Switch company for current user
    pub async fn switch_company(
        &self,
        user_id: Uuid,
        company_id: Uuid,
        _state: Arc<AppState>,
    ) -> Result<String, AppError> {
        let tenant_db: Option<String> = self
            .repo
            .get_company_for_switch(user_id, company_id)
            .await?;

        if tenant_db.is_none() {
            return Err(AppError::Unauthorized(
                "Not a member of this company".into(),
            ));
        }

        // Get Tenant Secret From Secrets Table
        let tenant_url_option: Option<String> = self.repo.get_tenant_url(Some(company_id)).await?;
        let tenant_url: &str = tenant_url_option
            .as_deref()
            .ok_or(AppError::Internal("Tenant Url missing".to_string()))?;

        // Lazy Load ensures company database is loaded
        Self::ensure_tenant_pool(&_state, company_id, tenant_url.as_ref()).await?;
        self.generate_token(user_id, Some(company_id), tenant_db)
    }

    /// Pure function for generating login and signup tokens
    pub fn generate_token(
        &self,
        user_id: Uuid,
        company_uuid: Option<Uuid>,
        tenant_db_name: Option<String>,
    ) -> Result<String, AppError> {
        let claims: JwtClaims = JwtClaims {
            sub: user_id,
            company_id: company_uuid,
            tenant_db: tenant_db_name.clone(),
            exp: (Utc::now() + Duration::weeks(24)).timestamp() as usize,
        };

        println!("Company ID in Gen Token: {:?}", claims.company_id);
        println!("Company db in Gen Token: {:?}", claims.tenant_db);

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Internal("Failed to generate JWT".into()))
    }

    /// # Tenant Pool Loader
    /// Method ensures company pool is loaded after successfull authentication
    async fn ensure_tenant_pool(
        state: &Arc<AppState>,
        company_id: Uuid,
        tenant_url: &str,
    ) -> Result<(), AppError> {
        if state.tenant_pools.get(&company_id).is_some() {
            return Ok(());
        }

        let pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(10)
            .connect(tenant_url)
            .await?;

        state.tenant_pools.insert(company_id, pool);

        println!("⚡ Lazy-loaded tenant pool for company {}", company_id);
        Ok(())
    }
}
