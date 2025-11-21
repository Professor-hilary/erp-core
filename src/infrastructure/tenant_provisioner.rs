use rand::{Rng, distr::Alphanumeric};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TenantProvisioner;

impl TenantProvisioner {
    /// Create a tenant database for a new company
    /// Returns: pool to tenant DB (ready for use)
    pub async fn create_tenant_db(
        master_pool: &PgPool,
        user_id: Uuid,
        company_name: &str,
        tenant_base_url: &str, // "postgres://user:pass@host:port/"
    ) -> Result<PgPool, sqlx::Error> {
        // Slug & tenant DB name
        let slug = company_name.to_lowercase().replace(' ', "_");
        let tenant_db_name = format!("tenant_{}_{}", user_id.simple(), slug);

        // Random password for role
        let password: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // SQL: create DB and role
        let create_db_sql = format!(r#"CREATE DATABASE "{}""#, tenant_db_name);
        let create_role_sql = format!(
            r#"CREATE ROLE "{}" WITH LOGIN PASSWORD '{}'"#,
            tenant_db_name, password
        );
        let grant_sql = format!(
            r#"GRANT ALL ON DATABASE "{}" TO "{}""#,
            tenant_db_name, tenant_db_name
        );

        // Run on master pool
        let mut tx = master_pool.begin().await?;
        sqlx::query(&create_db_sql).execute(&mut *tx).await?;
        sqlx::query(&create_role_sql).execute(&mut *tx).await?;
        sqlx::query(&grant_sql).execute(&mut *tx).await?;
        tx.commit().await?;

        // Connect to tenant DB
        let tenant_url = format!("{}{}", tenant_base_url, tenant_db_name);
        let tenant_pool = PgPool::connect(&tenant_url).await?;

        // Run tenant migrations
        sqlx::migrate!("./migrations/tenant")
            .run(&tenant_pool)
            .await?;

        Ok(tenant_pool)
    }
}
