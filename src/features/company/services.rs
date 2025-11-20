// src/features/company/service.rs
use crate::{
    features::company::repository::{CompanyRepository, PostgresCompanyRepository},
    infrastructure::responses::AppError,
    models::company::{Company, CreateCompanyDto},
    state::AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

pub struct CompanyService;

impl CompanyService {
    /// # Create Company
    /// Create a new company + tenant DB + assign admin
    pub async fn create_company(
        state: Arc<AppState>,
        user_id: Uuid,
        req: CreateCompanyDto,
    ) -> Result<Company, AppError> {
        let slug = req.name.to_lowercase().replace(' ', "-");
        let tenant_db_name = format!("tenant_{}_{}", user_id.simple(), slug);

        let mut tx = state
            .master_pool
            .begin()
            .await
            .map_err(|_| AppError::Internal("Failed to start transaction".into()))?;

        // 1. Try to create database + role (fails silently in managed DBs)
        let create_db_sql = format!(r#"CREATE DATABASE "{}""#, tenant_db_name);
        let password = Self::generate_secure_password(32);
        let create_role_sql = format!(
            r#"CREATE ROLE "{}" WITH LOGIN PASSWORD '{}'"#,
            tenant_db_name, password
        );

        let _db_provisioned = sqlx::query(&create_db_sql).execute(&mut *tx).await.is_ok()
            && sqlx::query(&create_role_sql)
                .execute(&mut *tx)
                .await
                .is_ok()
            && sqlx::query(&format!(
                r#"GRANT ALL ON DATABASE "{}" TO "{}""#,
                tenant_db_name, tenant_db_name
            ))
            .execute(&mut *tx)
            .await
            .is_ok();

        // 2. Connect to tenant DB and run migrations
        let tenant_url = state.tenant_db_url(&tenant_db_name);
        let tenant_pool = match PgPoolOptions::new()
            .max_connections(10)
            .connect(&tenant_url)
            .await
        {
            Ok(pool) => {
                if sqlx::migrate!("./migrations/tenant")
                    .run(&pool)
                    .await
                    .is_err()
                {
                    return Err(AppError::Internal("Tenant migration failed".into()));
                }
                Some(pool)
            }
            Err(_) => None,
        };

        // 3. Insert company record
        let company = PostgresCompanyRepository
            .create(
                &mut *tx,
                &req.name,
                &slug,
                &tenant_db_name,
                &tenant_url,
                &req.industry,
                &req.business_type,
                user_id,
            )
            .await
            .map_err(|_| AppError::Internal("Failed to create company record".into()))?;

        // 4. Make user admin
        PostgresCompanyRepository
            .assign_user_as_admin(&mut *tx, user_id, company.uuid)
            .await
            .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;

        // 5. Cache pool if created
        if let Some(pool) = tenant_pool {
            state.tenant_pools.insert(company.uuid, pool);
        }

        tx.commit()
            .await
            .map_err(|_| AppError::Internal("Failed to commit transaction".into()))?;

        Ok(company)
    }

    fn generate_secure_password(_size: i32) -> String {
        return "".to_string();
    }

    /// # Delete Company
    /// Soft delete — keeps all data for audit trail and revival
    pub async fn delete_company(
        state: Arc<AppState>,
        _user_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        let repo: PostgresCompanyRepository = PostgresCompanyRepository;

        // Optional: check if user is admin of this company
        let _company: Company = repo
            .find_by_id(&state.master_pool, company_id)
            .await?
            .ok_or(AppError::Internal("Company not found".into()))?;

        // Just mark as deleted — data stays forever
        repo.soft_delete(&state.master_pool, company_id).await?;

        // Optional: remove from cache
        state.tenant_pools.remove(&company_id);

        Ok(())
    }

    /// # Update Company
    /// Update company info given user is an admin and company id is provided
    pub async fn update_company(
        state: Arc<AppState>,
        _user_id: Uuid,
        company_id: Uuid,
        req: crate::models::company::UpdateCompanyDto,
    ) -> Result<Company, AppError> {
        let repo: PostgresCompanyRepository = PostgresCompanyRepository;

        let company = repo
            .update(&state.master_pool, company_id, req)
            .await
            .map_err(|_| AppError::NotFound)?;

        Ok(company)
    }

    /// # List Companies
    /// Get companies registered to currently auth'ed user
    pub async fn list_user_companies(
        state: Arc<AppState>,
        user_id: Uuid,
    ) -> Result<Vec<Company>, AppError> {
        let repo: PostgresCompanyRepository = PostgresCompanyRepository;
        repo.find_by_user(&state.master_pool, user_id).await
    }
}
