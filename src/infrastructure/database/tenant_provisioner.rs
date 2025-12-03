// src/infrastructure/database/tenant_provisioner.rs
use crate::infrastructure::errors::AppError;
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

pub struct TenantProvisioner;

impl TenantProvisioner {
    pub async fn create_tenant_db(
        master_pool: &PgPool,
        _user_id: Uuid,
        company_name: &str,
    ) -> Result<(PgPool, String), AppError> {
        let slug: String = company_name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .take(30)
            .collect::<String>();

        let db_name: String = format!("tenant_{}", slug);
        let role_name: String = format!("role_{}", slug);
        let password: String = rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Helper for idempotent steps
        macro_rules! ignore_already_exists {
            ($query:expr) => {{
                let result = sqlx::query($query).execute(master_pool).await;
                match result {
                    Ok(_) => Ok(()),
                    Err(e) if e.to_string().contains("already exists") => Ok(()),
                    Err(e) => Err(AppError::Internal(format!("DB setup failed: {}", e))),
                }
            }};
        }

        // ---------------------------------------------------------
        // 1. Create database + role + privileges
        ignore_already_exists!(&format!("CREATE DATABASE \"{}\"", db_name))?;

        ignore_already_exists!(&format!(
            "CREATE ROLE \"{}\" WITH LOGIN PASSWORD '{}' INHERIT",
            role_name, password
        ))?;

        sqlx::query(&format!("ALTER ROLE \"{}\" INHERIT", role_name))
            .execute(master_pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to alter role inherit: {}", e)))?;

        ignore_already_exists!(&format!("GRANT \"{}\" TO master_user", role_name))?;

        ignore_already_exists!(&format!(
            "ALTER DATABASE \"{}\" OWNER TO {}",
            db_name, role_name
        ))?;

        ignore_already_exists!(&format!(
            "GRANT ALL PRIVILEGES ON DATABASE \"{}\" TO {}",
            db_name, role_name
        ))?;

        // ---------------------------------------------------------
        // 2. Connect directly as tenant role — no SET ROLE needed
        let tenant_url: String = format!(
            "postgres://{}:{}@localhost:5433/{}",
            role_name, password, db_name
        );
        let tenant_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(10)
            .connect(&tenant_url)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to connect to tenant DB {}: {}", db_name, e))
            })?;

        // ---------------------------------------------------------
        // 3. Run tenant migrations
        sqlx::migrate!("./migrations/tenant")
            .run(&tenant_pool)
            .await
            .map_err(|e| AppError::Internal(format!("Tenant migrations failed: {}", e)))?;

        Ok((tenant_pool, tenant_url))
    }
}
