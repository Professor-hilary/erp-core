use super::actor::Actor;
use crate::infrastructure::redis_authz::SharedAuthzCache;
use crate::interface::api::errors::AppError;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpsertMembershipRequest {
    pub user_uuid: Uuid,
    pub department_uuid: Option<Uuid>,
    pub is_tenant_admin: Option<bool>,
    pub status: Option<String>,
    pub roles: Option<Vec<String>>,
    pub modules: Option<Vec<String>>,
}

pub struct RbacService;

impl RbacService {
    pub async fn bootstrap_admin(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query("SELECT rbac.bootstrap_tenant_admin($1)")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(format!("RBAC bootstrap failed: {e}")))?;
        Ok(())
    }

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
            return Err(AppError::Unauthorized(
                "User membership is not active".into(),
            ));
        }

        let roles: Vec<(String,)> =
            sqlx::query_as(r#"SELECT role_code FROM rbac.user_roles WHERE user_uuid = $1"#)
                .bind(user_id)
                .fetch_all(pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

        let modules: Vec<(String,)> =
            sqlx::query_as(r#"SELECT module FROM rbac.user_module_grants WHERE user_uuid = $1"#)
                .bind(user_id)
                .fetch_all(pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

        let permissions: Vec<(String,)> =
            sqlx::query_as(r#"SELECT permission_code FROM rbac.effective_permissions($1)"#)
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

    pub async fn list_roles(pool: &PgPool) -> Result<Vec<RoleRow>, AppError> {
        sqlx::query_as::<_, RoleRow>(
            r#"SELECT code, name, description, is_system FROM rbac.roles ORDER BY code"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn list_permissions(pool: &PgPool) -> Result<Vec<PermissionRow>, AppError> {
        sqlx::query_as::<_, PermissionRow>(
            r#"SELECT code, module, description FROM rbac.permissions ORDER BY module, code"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn list_memberships(pool: &PgPool) -> Result<Vec<MembershipListRow>, AppError> {
        sqlx::query_as::<_, MembershipListRow>(
            r#"
            SELECT m.user_uuid, m.department_uuid, m.is_tenant_admin, m.authz_version, m.status,
                   COALESCE(
                     (SELECT array_agg(ur.role_code ORDER BY ur.role_code)
                      FROM rbac.user_roles ur WHERE ur.user_uuid = m.user_uuid),
                     ARRAY[]::text[]
                   ) AS roles,
                   COALESCE(
                     (SELECT array_agg(um.module ORDER BY um.module)
                      FROM rbac.user_module_grants um WHERE um.user_uuid = m.user_uuid),
                     ARRAY[]::text[]
                   ) AS modules
            FROM rbac.user_memberships m
            ORDER BY m.created_at
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn upsert_membership(
        pool: &PgPool,
        cache: &Option<SharedAuthzCache>,
        company_id: Uuid,
        body: UpsertMembershipRequest,
    ) -> Result<(), AppError> {
        let is_admin = body.is_tenant_admin.unwrap_or(false);
        let status = body.status.as_deref().unwrap_or("active");
        if !matches!(status, "active" | "disabled" | "invited") {
            return Err(AppError::BadRequest(
                "status must be active|disabled|invited".into(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO rbac.user_memberships (user_uuid, department_uuid, is_tenant_admin, status)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_uuid) DO UPDATE SET
              department_uuid = EXCLUDED.department_uuid,
              is_tenant_admin = EXCLUDED.is_tenant_admin,
              status = EXCLUDED.status,
              authz_version = rbac.user_memberships.authz_version + 1,
              updated_at = now()
            "#,
        )
        .bind(body.user_uuid)
        .bind(body.department_uuid)
        .bind(is_admin)
        .bind(status)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(roles) = body.roles.as_ref() {
            Self::set_roles(pool, cache, company_id, body.user_uuid, roles).await?;
        }
        if let Some(modules) = body.modules.as_ref() {
            Self::set_modules(pool, cache, company_id, body.user_uuid, modules).await?;
        } else {
            Self::invalidate_actor(cache, company_id, body.user_uuid).await?;
        }
        Ok(())
    }

    pub async fn set_roles(
        pool: &PgPool,
        cache: &Option<SharedAuthzCache>,
        company_id: Uuid,
        user_uuid: Uuid,
        roles: &[String],
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO rbac.user_memberships (user_uuid, status)
            VALUES ($1, 'active')
            ON CONFLICT (user_uuid) DO NOTHING
            "#,
        )
        .bind(user_uuid)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sqlx::query("DELETE FROM rbac.user_roles WHERE user_uuid = $1")
            .bind(user_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        for r in roles {
            sqlx::query(
                r#"INSERT INTO rbac.user_roles (user_uuid, role_code) VALUES ($1, $2)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(user_uuid)
            .bind(r)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid role '{r}': {e}")))?;
        }
        sqlx::query("SELECT rbac.bump_authz_version($1)")
            .bind(user_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Self::invalidate_actor(cache, company_id, user_uuid).await?;
        Ok(())
    }

    pub async fn set_modules(
        pool: &PgPool,
        cache: &Option<SharedAuthzCache>,
        company_id: Uuid,
        user_uuid: Uuid,
        modules: &[String],
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO rbac.user_memberships (user_uuid, status)
            VALUES ($1, 'active')
            ON CONFLICT (user_uuid) DO NOTHING
            "#,
        )
        .bind(user_uuid)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        sqlx::query("DELETE FROM rbac.user_module_grants WHERE user_uuid = $1")
            .bind(user_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        for m in modules {
            sqlx::query(
                r#"INSERT INTO rbac.user_module_grants (user_uuid, module) VALUES ($1, $2)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(user_uuid)
            .bind(m)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
        sqlx::query("SELECT rbac.bump_authz_version($1)")
            .bind(user_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Self::invalidate_actor(cache, company_id, user_uuid).await?;
        Ok(())
    }

    pub async fn set_department(
        pool: &PgPool,
        cache: &Option<SharedAuthzCache>,
        company_id: Uuid,
        user_uuid: Uuid,
        department_uuid: Option<Uuid>,
    ) -> Result<(), AppError> {
        let res = sqlx::query(
            r#"
            UPDATE rbac.user_memberships
            SET department_uuid = $2,
                authz_version = authz_version + 1,
                updated_at = now()
            WHERE user_uuid = $1
            "#,
        )
        .bind(user_uuid)
        .bind(department_uuid)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("Membership not found".into()));
        }
        Self::invalidate_actor(cache, company_id, user_uuid).await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MembershipRow {
    user_uuid: Uuid,
    department_uuid: Option<Uuid>,
    is_tenant_admin: bool,
    authz_version: i64,
    status: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct RoleRow {
    pub code: String,
    pub name: String,
    pub description: String,
    pub is_system: bool,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct PermissionRow {
    pub code: String,
    pub module: String,
    pub description: String,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct MembershipListRow {
    pub user_uuid: Uuid,
    pub department_uuid: Option<Uuid>,
    pub is_tenant_admin: bool,
    pub authz_version: i64,
    pub status: String,
    pub roles: Vec<String>,
    pub modules: Vec<String>,
}
