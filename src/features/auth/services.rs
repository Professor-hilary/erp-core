// src/features/auth/services.rs
use crate::{
    features::auth::repository::UserRepository,
    infrastructure::errors::AppError,
    models::{
        dto::JwtClaims,
        user::{CreateUser, LoginUser, User, UserCompany},
    },
    state::{AppState, PeriodInfo},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::{PgPool, postgres::PgPoolOptions};
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
    ) -> Result<(User, Option<UserCompany>, String), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::BadRequest("Email and password required".into()));
        }

        let password_hash: String = hash(&user.password, DEFAULT_COST)
            .map_err(|_| AppError::Internal("Failed to hash password".into()))?;

        let created_user: User = self.repo.create(&user.email, &password_hash).await?;

        // this will automatically return None because at registration, no company is created
        let user_company: Option<UserCompany> =
            self.repo.get_user_company(created_user.uuid).await?;

        let (company_id, tenant_db_name) = match &user_company {
            Some(uc) => (uc.company_id, uc.tenant_db_name.clone()),
            None => (None, None),
        };

        let token: String = self.generate_token(created_user.uuid, company_id, tenant_db_name)?;

        Ok((created_user, user_company, token))
    }

    /// Sign in user from master database, update tenant pools in state
    pub async fn login(
        &self,
        user: &LoginUser,
        state: Arc<AppState>,
    ) -> Result<(User, Option<UserCompany>, String), AppError> {
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

        let user_company: Option<UserCompany> = self.repo.get_user_company(db_user.uuid).await?;

        // Only load tenant pool if company exists AND has valid tenant URL
        if let Some(uc) = &user_company {
            if let Some(company_uuid) = uc.company_id {
                match self.repo.get_tenant_url(Some(company_uuid)).await? {
                    Some(tenant_url) => {
                        // Valid company setup - load pool
                        Self::ensure_tenant_pool(&state, company_uuid, &tenant_url).await?;
                    }
                    None => {
                        // Company exists but incomplete set up - treat as no company
                        if cfg!(debug_assertions) {
                            println!(
                                "Company {} exists but missing tenant URL - treating as no company",
                                company_uuid
                            );
                        }
                    }
                }
            }
        }

        // Safe unwrapping with match
        let (company_id, tenant_db_name) = match &user_company {
            Some(uc) => (uc.company_id, uc.tenant_db_name.clone()),
            None => (None, None),
        };

        let token: String = self.generate_token(db_user.uuid, company_id, tenant_db_name)?;

        Ok((db_user, user_company, token))
    }

    pub async fn fetch_current_period(
        tenant_pool: PgPool,
        company_id: Uuid,
    ) -> Result<PeriodInfo, AppError> {
        #[derive(sqlx::FromRow)]
        struct PeriodRow {
            uuid: Uuid,
            start_date: chrono::NaiveDate,
            end_date: chrono::NaiveDate,
            is_locked: Option<bool>,
        }

        let row_opt: Option<PeriodRow> = sqlx::query_as::<_, PeriodRow>(
            r#"
                SELECT uuid, start_date, end_date, is_locked
                FROM accounting.financial_periods
                WHERE company_id = $1
                AND is_open = true
                ORDER BY start_date DESC
                LIMIT 1
                "#,
        )
        .bind(company_id)
        .fetch_optional(&tenant_pool)
        .await
        .map_err(|e| AppError::Database(e))?; // ← use anyhow or Box<dyn Error>

        let row = row_opt.ok_or_else(|| {
            AppError::Internal("No open financial period for this company".into())
        })?;

        Ok(PeriodInfo {
            _uuid: row.uuid,
            start_date: row.start_date,
            end_date: row.end_date,
            is_locked: row.is_locked.unwrap_or(false),
        })
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
            exp: (Utc::now() + Duration::weeks(54)).timestamp() as usize,
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

        state.tenant_pools.insert(company_id, pool.clone());

        let period_info: PeriodInfo = Self::fetch_current_period(pool, company_id).await?; // your helper fn

        state.period_cache.insert(company_id, period_info).await;

        println!("⚡ Lazy-loaded tenant pool for company {}", company_id);
        Ok(())
    }
}
