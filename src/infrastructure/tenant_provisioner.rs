// src/infrastructure/tenant_provisioner.rs
use rand::{Rng, distr::Alphanumeric, rng};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

pub struct TenantProvisioner;

impl TenantProvisioner {
    pub async fn create_tenant_db(
        master_pool: &PgPool,
        user_id: Uuid,
        company_name: &str,
        tenant_base_url: &str, // e.g. "postgres://myapp:mysecret@localhost:5432/"
    ) -> Result<PgPool, sqlx::Error> {
        let slug = company_name
            .to_lowercase()
            .replace(' ', "_")
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>();

        let db_name = format!("tenant_{}_{}", user_id.simple(), slug);
        let role_name = &db_name; // 1:1 role per tenant
        let password: String = rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Note: CREATE DATABASE cannot be in a transaction block
        sqlx::query(&format!(r#"CREATE DATABASE "{}""#, db_name))
            .execute(master_pool)
            .await?;

        sqlx::query(&format!(
            r#"CREATE ROLE "{}" WITH LOGIN PASSWORD '{}'"#,
            role_name, password
        ))
        .execute(master_pool)
        .await?;

        sqlx::query(&format!(
            r#"GRANT ALL PRIVILEGES ON DATABASE "{}" TO "{}""#,
            db_name, role_name
        ))
        .execute(master_pool)
        .await?;

        // Connect using the new role (more secure than reusing master creds)
        let tenant_url = format!("{}{}?user={}&password={}", tenant_base_url, db_name, role_name, password);
        let tenant_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&tenant_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations/tenant")
            .run(&tenant_pool)
            .await?;

        Ok(tenant_pool)
    }
}
