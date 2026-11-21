use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    features::rbac::Actor,
    interface::api::errors::AppError,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedTenant {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub tenant_pool: sqlx::PgPool,
    pub current_period_start: NaiveDate,
    pub current_period_end: NaiveDate,
    pub period_is_locked: bool,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()))
    }
}

impl<S> FromRequestParts<S> for AuthenticatedTenant
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedTenant>()
            .cloned()
            .ok_or_else(|| {
                AppError::Unauthorized(
                    "No company selected. Create or switch to a company first".into(),
                )
            })
    }
}

impl<S> FromRequestParts<S> for Actor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Actor>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("Authorization context missing".into()))
    }
}
