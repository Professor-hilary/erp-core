// src/infrastructure/tenant_provisioner.rs
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use uuid::Uuid;

use crate::infrastructure::responses::AppError;
pub struct TenantProvisioner;

impl TenantProvisioner {
    pub async fn create_tenant_db(
        master_pool: &PgPool,
        user_id: Uuid,
        company_name: &str,
        //_tenant_base_url: &str,
    ) -> Result<PgPool, sqlx::Error> {
        let super_psql_url =
            env::var("POSTGRES_SUPER_URL").expect("POSTGRES_SUPER_URL missing");

        let _super_pool = PgPool::connect(&super_psql_url)
            .await
            .map_err(|e| format!("Failed to connect as superuser: {e}"));

        let s_pool = _super_pool.unwrap();

        let slug: String = company_name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let db_name: String = format!("tenant_{}_{}", user_id.simple(), slug);
        let safe_slug: String = slug.chars().take(30).collect::<String>();
        let role_name: String = format!("tenant_{}_{}", &user_id.simple(), safe_slug);

        let password: String = rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Note: CREATE DATABASE cannot be in a transaction block
        let _ = sqlx::query(&format!(r#"CREATE DATABASE "{}""#, db_name))
            .execute(master_pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()));

        let _ = sqlx::query(&format!(
            r#"CREATE ROLE "{}" WITH LOGIN PASSWORD '{}'"#,
            role_name, password
        ))
        .execute(master_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()));

        let _ = sqlx::query(&format!(
            r#"GRANT ALL PRIVILEGES ON DATABASE "{}" TO "{}""#,
            db_name, role_name
        ))
        .execute(master_pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()));

        // Connect using the new role (more secure than reusing master creds)
        let tenant_url = format!(
            "postgres://{}:{}@localhost:5432/{}",
            role_name, password, db_name
        );
        let tenant_pool: sqlx::Pool<sqlx::Postgres> = PgPoolOptions::new()
            .max_connections(10)
            .connect(&tenant_url)
            .await?;

        // Run migrations
        let _ = sqlx::migrate!("./migrations/tenant")
            .run(&tenant_pool)
            .await
            .map_err(|_| AppError::Internal("Tenant migration failed".into()));

        Ok(tenant_pool)
    }
}
