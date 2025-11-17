// src/features/auth/services.rs
use std::sync::Arc;
use crate::errors::AppError;
use crate::features::auth::repository::UserRepository;
use crate::models::dto::JwtClaims;
use crate::models::user::{CreateUser, LoginUser, User};
use crate::routes::AppState;
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

pub struct AuthService<R: UserRepository> {
    repo: R,
    state: Arc<AppState>,
}

impl<R: UserRepository> AuthService<R> {
    pub fn new(repo: R, state: Arc<AppState>) -> Self {
        Self { repo, state }
    }

    pub async fn register(&self, user: &CreateUser) -> Result<(User, String), AppError> {
        if user.email.is_empty() || user.password.is_empty() {
            return Err(AppError::Validation("Email and password required".into()));
        }

        let password_hash = hash(&user.password, DEFAULT_COST)
            .map_err(|_| AppError::Validation("Hash error".into()))?;

        let created_user = self
            .repo
            .create(&self.state.master_pool, &user.email, &password_hash)
            .await?;

        // Generate JWT immediately
        let claims = JwtClaims {
            sub: created_user.id,
            exp: (Utc::now() + Duration::weeks(4)).timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Auth("Token generation failed".into()))?;

        Ok((created_user, token))
    }

    pub async fn login(&self, user: &LoginUser) -> Result<String, AppError> {
        // Returns JWT
        let db_user = self
            .repo
            .find_by_email(&self.state.master_pool, &user.email)
            .await?
            .ok_or(AppError::Auth("User not found".to_string()))?;

        if !verify(&user.password, &db_user.password_hash)
            .map_err(|_| AppError::Auth("Invalid credentials".to_string()))?
        {
            return Err(AppError::Auth("Invalid credentials".to_string()));
        }

        let claims = JwtClaims {
            sub: db_user.id,
            exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.state.jwt_secret.as_ref()),
        )
        .map_err(|_| AppError::Auth("Token generation failed".to_string()))?;
        Ok(token)
    }
}
