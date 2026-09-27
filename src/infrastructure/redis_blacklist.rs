// src/infrastructure/redis_blacklist.rs
use crate::interface::api::errors::AppError;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use uuid::Uuid;

const KEY_PREFIX: &str = "revoked:jti:";

#[derive(Clone, Debug)]
pub struct TokenBlacklist {
    conn: ConnectionManager,
}

impl TokenBlacklist {
    /// Minimal health check - fails if Redis is unreachable
    pub async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Redis PING failed: {e}")))?;

        if pong != "PONG" {
            return Err(AppError::Internal(format!(
                "Redis PING unexpected reply: {pong}"
            )));
        }

        Ok(())
    }

    pub async fn connect(redis_url: &str) -> Result<Self, AppError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::Internal(format!("Redis client error: {e}")))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Internal(format!("Redis connection error: {e}")))?;
        Ok(Self { conn })
    }

    /// Blacklist jti until ttl_secs (usually remaining token lifetime).
    pub async fn revoke(&self, jti: Uuid, ttl_secs: u64) -> Result<(), AppError> {
        if ttl_secs == 0 {
            return Ok(());
        }
        let mut conn: ConnectionManager = self.conn.clone();
        conn.set_ex::<_, _, ()>(format!("{KEY_PREFIX}{jti}"), "1", ttl_secs)
            .await
            .map_err(|e| AppError::Internal(format!("Redis SETEX failed: {e}")))?;
        tracing::debug!(%jti, ttl_secs, "token jti blacklisted");
        Ok(())
    }

    pub async fn is_revoked(&self, jti: Uuid) -> Result<bool, AppError> {
        let mut conn = self.conn.clone();
        let exists: bool = conn
            .exists(format!("{KEY_PREFIX}{jti}"))
            .await
            .map_err(|e| AppError::Internal(format!("Redis EXISTS failed: {e}")))?;
        Ok(exists)
    }
}

pub type SharedBlacklist = Arc<TokenBlacklist>;
