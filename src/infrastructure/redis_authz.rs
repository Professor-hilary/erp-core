//! Authz Actor cache in Redis — one GET per request on hit; invalidate on role change.
use crate::{features::rbac::Actor, interface::api::errors::AppError};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

const KEY_PREFIX: &str = "authz:actor:";
/// Default TTL; role changes invalidate explicitly. TTL is a safety net.
const DEFAULT_TTL_SECS: u64 = 900;

#[derive(Clone, Debug)]
pub struct AuthzCache {
    conn: ConnectionManager,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActorWire {
    user_id: Uuid,
    company_id: Uuid,
    is_tenant_admin: bool,
    department_uuid: Option<Uuid>,
    authz_version: i64,
    modules: Vec<String>,
    permissions: Vec<String>,
    roles: Vec<String>,
}

impl From<&Actor> for ActorWire {
    fn from(a: &Actor) -> Self {
        Self {
            user_id: a.user_id,
            company_id: a.company_id,
            is_tenant_admin: a.is_tenant_admin,
            department_uuid: a.department_uuid,
            authz_version: a.authz_version,
            modules: a.modules.iter().cloned().collect(),
            permissions: a.permissions.iter().cloned().collect(),
            roles: a.roles.iter().cloned().collect(),
        }
    }
}

impl From<ActorWire> for Actor {
    fn from(w: ActorWire) -> Self {
        Self {
            user_id: w.user_id,
            company_id: w.company_id,
            is_tenant_admin: w.is_tenant_admin,
            department_uuid: w.department_uuid,
            authz_version: w.authz_version,
            modules: w.modules.into_iter().collect(),
            permissions: w.permissions.into_iter().collect(),
            roles: w.roles.into_iter().collect(),
        }
    }
}

impl AuthzCache {
    pub async fn connect(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Redis authz client: {e}")))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Redis authz connect: {e}")))?;
        Ok(Self { conn })
    }

    fn key(company_id: Uuid, user_id: Uuid) -> String {
        format!("{KEY_PREFIX}{company_id}:{user_id}")
    }

    pub async fn get(&self, company_id: Uuid, user_id: Uuid) -> Result<Option<Actor>, AppError> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn
            .get(Self::key(company_id, user_id))
            .await
            .map_err(|e| AppError::Internal(format!("Redis authz GET: {e}")))?;
        let Some(s) = raw else {
            return Ok(None);
        };
        let wire: ActorWire = serde_json::from_str(&s)
            .map_err(|e| AppError::Internal(format!("Redis authz decode: {e}")))?;
        Ok(Some(wire.into()))
    }

    pub async fn set(&self, actor: &Actor) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let wire = ActorWire::from(actor);
        let payload = serde_json::to_string(&wire)
            .map_err(|e| AppError::Internal(format!("Redis authz encode: {e}")))?;
        conn.set_ex::<_, _, ()>(
            Self::key(actor.company_id, actor.user_id),
            payload,
            DEFAULT_TTL_SECS,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Redis authz SETEX: {e}")))?;
        Ok(())
    }

    pub async fn invalidate(&self, company_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let _: i64 = conn
            .del(Self::key(company_id, user_id))
            .await
            .map_err(|e| AppError::Internal(format!("Redis authz DEL: {e}")))?;
        Ok(())
    }
}

pub type SharedAuthzCache = Arc<AuthzCache>;
