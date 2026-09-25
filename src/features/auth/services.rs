// src/features/auth/services.rs
use crate::{
    features::auth::repository::UserRepository,
    interface::api::errors::AppError,
    models::{
        dto::JwtClaims,
        user::{CreateUser, LoginUser, User, UserCompany},
    },
    state::{AppState, PeriodInfo},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
// use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use password_hash::SaltString;
use rand_core::OsRng;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService<R: UserRepository> {
    repo: R,
    state: Arc<AppState>,
}

const ACCESS_EXPIRY: Duration = Duration::minutes(30);
const REFRESH_EXPIRY: Duration = Duration::days(30);

impl<R: UserRepository> AuthService<R> {
    pub fn new(repo: R, state: Arc<AppState>) -> Self {
        Self { repo, state }
    }

    /// Sign up user into master database, update tenant pools in state
    pub async fn register(
        &self,
        user: CreateUser,
        _state: Arc<AppState>,
    ) -> Result<(User, Option<UserCompany>, (String, String)), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::BadRequest("Email and password required".into()));
        }

        // let password_hash: String = hash(&user.password, DEFAULT_COST)
        //     .map_err(|_| AppError::Internal("Failed to hash password".into()))?;
        let password_hash: String = Self::hash_password(&user.password)
            .map_err(|_| AppError::Internal("Failed to hash password".into()))?;

        let created_user: User = self.repo.create(user, &password_hash).await?;

        // this will automatically return None because at registration, no company is created
        let user_company: Option<UserCompany> =
            self.repo.get_user_company(created_user.uuid).await?;

        let (company_id, tenant_db_name) = match &user_company {
            Some(uc) => (uc.company_id, uc.tenant_db_name.clone()),
            None => (None, None),
        };

        let token = self.generate_token_pair(created_user.uuid, company_id, tenant_db_name)?;

        Ok((created_user, user_company, token))
    }

    /// Sign in user from master database, update tenant pools in state
    pub async fn login(
        &self,
        user: &LoginUser,
        state: Arc<AppState>,
    ) -> Result<(User, Option<UserCompany>, (String, String)), AppError> {
        let db_user: User = self
            .repo
            .find_by_email(&user.email)
            .await?
            .ok_or(AppError::Unauthorized("Invalid credentials".into()))?;

        // if !verify(&user.password, &db_user.password_hash)
        //     .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?
        // {
        //     return Err(AppError::Unauthorized("Invalid credentials".into()));
        // }
        if !Self::verify_password(&user.password, &db_user.password_hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?
        {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        let user_company: Option<UserCompany> = self.repo.get_user_company(db_user.uuid).await?;

        // Only load tenant pool if company exists AND has valid tenant URL
        if let Some(uc) = &user_company
            && let Some(company_uuid) = uc.company_id
        {
            match self.repo.get_tenant_url(Some(company_uuid)).await? {
                Some(tenant_url) => {
                    // Valid company setup - load pool
                    if let Err(e) =
                        Self::ensure_tenant_pool(&state, company_uuid, &tenant_url).await
                    {
                        tracing::warn!("Tenant pool load failed: {:?}", e);
                    }
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

        // Safe unwrapping with match
        let (company_id, tenant_db_name) = match &user_company {
            Some(uc) => (uc.company_id, uc.tenant_db_name.clone()),
            None => (None, None),
        };

        let token = self.generate_token_pair(db_user.uuid, company_id, tenant_db_name)?;

        Ok((db_user, user_company, token))
    }

    pub async fn fetch_current_period(tenant_pool: PgPool) -> Result<PeriodInfo, AppError> {
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
                WHERE is_open = true
                ORDER BY start_date DESC
                LIMIT 1
                "#,
        )
        // .bind(company_id)
        .fetch_optional(&tenant_pool)
        .await
        .map_err(|e: sqlx::Error| AppError::Database(e))?;

        let row: PeriodRow = row_opt.ok_or_else(|| {
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
    ) -> Result<(String, String), AppError> {
        let tenant_db = self
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
        Self::ensure_tenant_pool(&_state, company_id, tenant_url).await?;
        self.generate_token_pair(user_id, Some(company_id), tenant_db)
    }

    pub fn generate_token_pair(
        &self,
        user_id: Uuid,
        company_uuid: Option<Uuid>,
        tenant_db_name: Option<String>,
    ) -> Result<(String, String), AppError> {
        let access = self.generate_token(
            user_id,
            company_uuid,
            tenant_db_name.clone(),
            "access",
            ACCESS_EXPIRY,
        )?;
        let refresh = self.generate_token(
            user_id,
            company_uuid,
            tenant_db_name,
            "refresh",
            REFRESH_EXPIRY,
        )?;

        Ok((access, refresh))
    }

    /// Pure function for generating login and signup tokens
    pub fn generate_token(
        &self,
        user_id: Uuid,
        company_uuid: Option<Uuid>,
        tenant_db_name: Option<String>,
        token_type: &str,
        lifetime: Duration,
    ) -> Result<String, AppError> {
        let claims: JwtClaims = JwtClaims {
            sub: user_id,
            company_id: company_uuid,
            tenant_db: tenant_db_name.clone(),
            // exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
            exp: (Utc::now() + lifetime).timestamp() as usize,
            token_type: token_type.to_string(),
            jti: Uuid::new_v4(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Internal("Failed to generate JWT".into()))
    }

    /// Pure function for generating login and signup tokens
    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<(String, String), AppError> {
        let data = decode::<JwtClaims>(
            refresh_token,
            &DecodingKey::from_secret(self.state.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".into()))?;

        if data.claims.token_type != "refresh" {
            return Err(AppError::Unauthorized("Not a refresh token".into()));
        }

        self.generate_token_pair(
            data.claims.sub,
            data.claims.company_id,
            data.claims.tenant_db,
        )
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

        let period_info: PeriodInfo = Self::fetch_current_period(pool).await?; // your helper fn

        state.period_cache.insert(company_id, period_info).await;

        println!("⚡ Lazy-loaded tenant pool for company {}", company_id);
        Ok(())
    }

    // Hashing (register)
    fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| AppError::Internal("Failed to hash password".into()))
    }

    // Verifying (login)
    fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}
