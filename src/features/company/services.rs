// src/features/company/service.rs
use crate::{
    features::company::repository::{CompanyRepository, PostgresCompanyRepository},
    infrastructure::{database::tenant_provisioner::TenantProvisioner, responses::AppError },
    models::{
        coa_entry::ChartOfAccountsEntry,
        company::{Company, CreateCompanyDto},
    },
    state::AppState,
};
use sqlx::PgPool;
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

        // Step 1: Start transaction on master DB (for company record + admin assignment)
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = state
            .master_pool
            .begin()
            .await
            .map_err(|_| AppError::Internal("Failed to start transaction".into()))?;

        // Step 2: Provision tenant database (this commits independently)
        // Note: We cannot keep this inside the same tx as master operations
        // because CREATE DATABASE cannot run inside a transaction block in PostgreSQL
        let tenant_pool = TenantProvisioner::create_tenant_db(
            &state.master_pool,
            user_id,
            &req.name,
            &state.tenant_config.base_url, // e.g. "postgres://app_user:app_pass@localhost:5432/"
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to provision tenant database: {}", e)))?;

        let tenant_db_name = format!("tenant_{}_{}", user_id.simple(), slug);
        let tenant_url = format!("{}{}", state.tenant_config.base_url, tenant_db_name);

        // Run migrations (already done inside TenantProvisioner, but safe to run again)
        sqlx::migrate!("./migrations/tenant")
            .run(&tenant_pool)
            .await
            .map_err(|_| AppError::Internal("Tenant migration failed".into()))?;

        // Step 3: Seed Chart of Accounts
        Self::seed_coa(&tenant_pool, &req.business_type)
            .await
            .map_err(|_| AppError::Internal("Failed to seed Chart of Accounts".into()))?;

        let repo = PostgresCompanyRepository;

        // Step 4: Insert company record into master DB
        let company = repo
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

        // Step 5: Assign user as admin + activate
        repo.assign_user_as_admin(&mut *tx, user_id, company.uuid)
            .await
            .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;

        repo.update_status(&mut *tx, company.uuid, "active")
            .await
            .map_err(|_| AppError::Internal("Failed to activate company".into()))?;

        // Step 6: Commit master transaction
        tx.commit()
            .await
            .map_err(|_| AppError::Internal("Failed to commit transaction".into()))?;

        // Step 7: Cache the tenant pool only after everything succeeds
        state.tenant_pools.insert(company.uuid, tenant_pool);

        Ok(company)
    }

    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
            .trim_matches('-')
            .replace("--", "-")
    }

    #[allow(dead_code)]
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

    /// # COA seed
    /// Seed default chart of accounts for a tenant based on company type
    pub async fn seed_coa(tenant_pool: &PgPool, business_type: &str) -> Result<(), AppError> {
        // Map business_type to file name (Adjust mapping as needed)
        let file_name = match business_type {
            "agriculture" => "coa_agriculture.json",
            "energy" => "coa_energy.json",
            "infotech" => "coa_infotechnology.json",
            "non_profit" => "coa_not_for_profit.json",
            "wholesome" => "coa_wholesale.json",
            "construction" => "coa_construction.json",
            "healthcare" => "coa_healthcare.json",
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
