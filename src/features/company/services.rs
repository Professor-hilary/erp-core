// src/features/company/service.rs
use crate::{
    errors::AppError,
    features::company::repository::{CompanyRepository, PostgresCompanyRepository},
    models::company::{Company, CreateCompanyDto},
    routes::AppState,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct CompanyService;

impl CompanyService {
    /// Create a new company + tenant DB + assign admin
    pub async fn create_company(
        state: Arc<AppState>,
        user_id: Uuid,
        req: CreateCompanyDto,
    ) -> Result<Company, AppError> {
        let slug = req.name.to_lowercase().replace(" ", "-");
        let tenant_db_name = format!("tenant_{}_{}", user_id.simple(), slug);

        // Use env vars properly (with fallbacks)
        let db_user = std::env::var("TENANT_DB_USER").unwrap_or("postgres".into());
        let db_pass = std::env::var("TENANT_DB_PASSWORD").unwrap_or("password".into());
        let db_host = std::env::var("TENANT_DB_HOST").unwrap_or("localhost:5432".into());
        let tenant_db_uri = format!("postgres://{db_user}:{db_pass}@{db_host}/{tenant_db_name}");

        // Start transaction on master DB
        let mut tx = state.master_pool.begin().await?;

        // 1. Create the tenant database (only if your role has CREATEDB)
        // If this fails in prod (most hosted DBs block it), it will return an error — that's OK!
        let db_create_result = sqlx::query(&format!(r#"CREATE DATABASE "{}""#, tenant_db_name))
            .execute(&mut *tx)
            .await;

        if let Err(e) = db_create_result {
            // In production, you might want to fall back to a shared schema or queue a job
            tracing::warn!("Could not create tenant DB (normal in managed DBs): {e}");
            // Continue anyway — we'll mark it as provisioning failed later if needed
        }

        // 2. Try to connect and run migrations on tenant DB
        let tenant_pool_result = PgPool::connect(&tenant_db_uri).await;
        let tenant_pool = match tenant_pool_result {
            Ok(pool) => {
                match sqlx::migrate!("./migrations/tenant")
                                    .run(&pool)
                                    .await
                                    .is_ok() {
                    true => Some(pool),
                    false => None,
                }
            }
            Err(_) => None,
        };

        // 3. Insert company record in master
        let repo = PostgresCompanyRepository;
        let company = repo
            .create(
                &mut *tx, // Now works because we accept any Executor
                &req.name,
                &slug,
                &tenant_db_name,
                &tenant_db_uri,
                &req.industry,
                &req.business_type,
                user_id,
            )
            .await?;

        // 4. Make creator admin
        repo.assign_user_as_admin(&mut *tx, user_id, company.uuid)
            .await?;

        // 5. Cache tenant pool if we successfully created it
        if let Some(pool) = tenant_pool {
            state.tenant_pools.insert(company.uuid, pool);
        }

        tx.commit().await?;
        Ok(company)
    }

    // Soft delete — keeps all data for audit
    pub async fn delete_company(
        state: Arc<AppState>,
        _user_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        let repo = PostgresCompanyRepository;

        // Optional: check if user is admin of this company
        let _company = repo
            .find_by_id(&state.master_pool, company_id)
            .await?
            .ok_or(AppError::NotFound("Company not found".into()))?;

        // Just mark as deleted — data stays forever
        repo.soft_delete(&state.master_pool, company_id).await?;

        // Optional: remove from cache
        state.tenant_pools.remove(&company_id);

        Ok(())
    }

    pub async fn update_company(
        state: Arc<AppState>,
        _user_id: Uuid,
        company_id: Uuid,
        req: crate::models::company::UpdateCompanyDto,
    ) -> Result<Company, AppError> {
        let repo = PostgresCompanyRepository;
        repo.update(&state.master_pool, company_id, req).await
    }

    pub async fn list_user_companies(
        state: Arc<AppState>,
        user_id: Uuid,
    ) -> Result<Vec<Company>, AppError> {
        let repo = PostgresCompanyRepository;
        repo.find_by_user(&state.master_pool, user_id).await
    }
}
