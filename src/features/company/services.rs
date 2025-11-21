// src/features/company/service.rs
use crate::{
    features::company::repository::{CompanyRepository, PostgresCompanyRepository},
    infrastructure::responses::AppError,
    models::{
        coa_entry::ChartOfAccountsEntry,
        company::{Company, CreateCompanyDto},
    },
    state::AppState,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{fs, sync::Arc};
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
        let slug = Self::slugify(&req.name);

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

        // 4a. Make user admin
        PostgresCompanyRepository
            .assign_user_as_admin(&mut *tx, user_id, company.uuid)
            .await
            .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;

        // 4b. Mark as active
        PostgresCompanyRepository
            .update_status(&mut *tx, company.uuid, "active")
            .await?;

        // 5. Cache pool if created
        if let Some(pool) = tenant_pool {
            state.tenant_pools.insert(company.uuid, pool);
        }

        tx.commit()
            .await
            .map_err(|_| AppError::Internal("Failed to commit transaction".into()))?;

        Ok(company)
    }

    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
            .trim_matches('-')
            .replace("--", "-")
    }

    fn generate_secure_password(size: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789-";

        let mut rng = rand::rng();
        (0..size)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
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

    /// Seed default chart of accounts for a tenant based on company type
    pub async fn seed_coa(tenant_pool: &PgPool, business_type: &str) -> Result<(), AppError> {
        // Map business_type to file name (Adjust mapping as needed)
        let file_name = match business_type {
            "agric" => "coa_agriculture.json",
            "energy" => "coa_energy.json",
            "infotech" => "coa_infotechnology.json",
            "nfp" => "coa_not_for_profit.json",
            "wholesome" => "coa_wholesale.json",
            "construction" => "coa_construction.json",
            "health" => "coa_healthcare.json",
            "logistics" => "coa_logistics.json",
            "services" => "coa_professional_services.json",
            "education" => "coa_education.json",
            "hospitality" => "coa_hospitality.json",
            "manufacturing" => "coa_manufacturing.json",
            "retail" => "coa_retail.json",
            _ => {
                return Err(AppError::Unauthorized(format!(
                    "Unknown business type: {}",
                    business_type
                )));
            }
        };

        let path = format!("./seed/{}", file_name);
        let contents = fs::read_to_string(&path)
            .map_err(|_| AppError::Internal(format!("Failed to reach COA file: {}", path)))?;

        let entries: Vec<ChartOfAccountsEntry> = serde_json::from_str(&contents)
            .map_err(|_| AppError::Internal(format!("Invalid JSON in COA file: {}", path)))?;

        // Insert into tenant DB
        for entry in entries {
            sqlx::query(
                r#"
                INSERT INTO accounting.accounts (code, name, category, parent_code, normal_balance, is_contra)
                VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (code) DO NOTHING
                "#
            )
            .bind(&entry.code)
            .bind(&entry.name)
            .bind(&entry.category)
            .bind(&entry.parent_code)
            .bind(&entry.normal_balance)
            .bind(&entry.is_contra)
            .execute(tenant_pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to insert COA entry: {}", e)))?;
        }

        Ok(())
    }
}
