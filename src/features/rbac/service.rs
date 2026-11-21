use super::actor::Actor;
use crate::infrastructure::redis_authz::SharedAuthzCache;
use crate::interface::api::errors::AppError;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

pub struct RbacService;

impl RbacService {
    /// Seed TenantAdmin membership for the company creator (tenant DB).
    pub async fn bootstrap_admin(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("SELECT rbac.bootstrap_tenant_admin($1)")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(format!("RBAC bootstrap failed: {e}")))?;
        Ok(())
    }

    /// Load effective Actor from tenant DB (call on cache miss only).
    pub async fn load_actor(
        pool: &PgPool,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Actor, AppError> {
        let membership = sqlx::query_as::<_, MembershipRow>(
            r#"
            SELECT user_uuid, department_uuid, is_tenant_admin, authz_version, status
            FROM rbac.user_memberships
            WHERE user_uuid = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let Some(m) = membership else {
            // No membership yet — empty actor (deny-by-default except we allow
            // bootstrap path to create admin). Callers treat as no grants.
            return Ok(Actor {
                user_id,
                company_id,
                is_tenant_admin: false,
                department_uuid: None,
                authz_version: 0,
                modules: HashSet::new(),
                permissions: HashSet::new(),
                roles: HashSet::new(),
            });
        };

        if m.status != "active" {
            return Err(AppError::Unauthorized("User membership is not active".into()));
        }

        let roles: Vec<(String,)> = sqlx::query_as(
            r#"SELECT role_code FROM rbac.user_roles WHERE user_uuid = $1"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let modules: Vec<(String,)> = sqlx::query_as(
            r#"SELECT module FROM rbac.user_module_grants WHERE user_uuid = $1"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let permissions: Vec<(String,)> = sqlx::query_as(
            r#"SELECT permission_code FROM rbac.effective_permissions($1)"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Actor {
            user_id,
            company_id,
            is_tenant_admin: m.is_tenant_admin,
            department_uuid: m.department_uuid,
            authz_version: m.authz_version,
            modules: modules.into_iter().map(|(x,)| x).collect(),
            permissions: permissions.into_iter().map(|(x,)| x).collect(),
            roles: roles.into_iter().map(|(x,)| x).collect(),
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MembershipRow {
    user_uuid: Uuid,
    department_uuid: Option<Uuid>,
    is_tenant_admin: bool,
    authz_version: i64,
    status: String,

    /// Cache-first Actor load. Handlers call this once per request via middleware.
    pub async fn resolve_actor(
        pool: &PgPool,
        cache: &Option<SharedAuthzCache>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Actor, AppError> {
        if let Some(c) = cache {
            if let Some(actor) = c.get(company_id, user_id).await? {
                return Ok(actor);
            }
        }
        let actor = Self::load_actor(pool, user_id, company_id).await?;
        if let Some(c) = cache {
            // Best-effort; do not fail the request if Redis SET fails
            if let Err(e) = c.set(&actor).await {
                tracing::warn!(error = %e, "authz cache set failed");
            }
        }
        Ok(actor)
    }

    pub async fn invalidate_actor(
        cache: &Option<SharedAuthzCache>,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        if let Some(c) = cache {
            c.invalidate(company_id, user_id).await?;
        }
        Ok(())
    }
}
