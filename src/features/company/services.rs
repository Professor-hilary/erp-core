// src/features/company/service.rs
use crate::features::{
    auth::{AuthService, repository::PostgresUserRepo},
    company::repository::{CompanyRepository, PostgresCompanyRepository},
};
use crate::models::{
    account::CoaTemplate,
    company::{Company, CreateCompanyDto},
};
use crate::{
    infrastructure::{database::tenant_provisioner::TenantProvisioner, errors::AppError},
    state::AppState,
};

use sqlx::{PgPool, Postgres};
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
    ) -> Result<(Company, String), AppError> {
        let slug: String = Self::slugify(&req.name);

        // Step 1: Start transaction on master DB (for company record + admin assignment)
        let mut tx: sqlx::Transaction<'_, Postgres> = state
            .master_pool
            .begin()
            .await
            .map_err(|_| AppError::Internal("Failed to start transaction".into()))?;

        // Step 2: Provision tenant database (this commits independently)
        // Note: We cannot keep this inside the same tx as master operations
        // because CREATE DATABASE cannot run inside a transaction block in PostgreSQL
        let (
            tenant_pool,
            tenant_db_uri,
            db_name,
            db_role,
            db_pass,
            db_host,
            db_port,
        ) = TenantProvisioner::create_tenant_db(&state.master_pool, user_id, &req.name).await?;

        let tenant_db_name: String = format!("tenant_{}", slug);

        // Step 3: Seed Chart of Accounts
        Self::seed_coa(&tenant_pool, &req.industry, &state.coa_seed_path)
            .await
            .map_err(|_| AppError::Internal("Failed to seed Chart of Accounts".into()))?;

        let repo: PostgresCompanyRepository = PostgresCompanyRepository;

        // Step 4: Insert company record into master DB
        let company: Company = repo
            .create(
                &mut *tx,
                &req.name,
                &slug,
                &tenant_db_name,
                &tenant_db_uri,
                &req.industry,
                &req.business_type,
                user_id,
            )
            .await
            .map_err(|_| AppError::Internal("Failed to create company record".into()))?;

        // Step 5: Assign user as admin + activate + update the secrets table
        repo.assign_user_as_admin(&mut *tx, user_id, company.uuid)
            .await
            .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;

        repo.update_status(&mut *tx, company.uuid, "active")
            .await
            .map_err(|_| AppError::Internal("Failed to activate company".into()))?;

        repo.update_company_secret(
            &mut *tx,
            user_id,
            &tenant_db_uri,
            company.uuid,
            db_name,
            db_role,
            db_pass,
            db_host,
            db_port,
        )
        .await
        .map_err(|_| AppError::Internal("Failed to update company secrets tables".into()))?;

        // Step 6: Commit master transaction
        tx.commit()
            .await
            .map_err(|_| AppError::Internal("Failed to commit transaction".into()))?;

        println!("Pool Size: {}", tenant_pool.size());

        // Step 7: Cache the tenant pool only after everything succeeds
        state.tenant_pools.insert(company.uuid, tenant_pool);

        // After caching tenant_pool and committing:
        let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());
        let auth_service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());

        let new_token: String = auth_service
            .generate_token(user_id, Some(company.uuid), Some(tenant_db_name))
            .map_err(|_| AppError::Internal("Failed to generate new token".into()))?;

        // Return both company + new token
        Ok((company, new_token))
    }

    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_ascii_alphanumeric(), "-")
            .trim_matches('-')
            .replace("--", "-")
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

        let company: Company = repo
            .update(&state.master_pool, company_id, req)
            .await
            .map_err(|e| AppError::NotFound(e.to_string()))?;

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

    /// Generate chart of accounts for selected company type
    pub async fn seed_coa(
        tenant_pool: &PgPool,
        business_type: &str,
        coa_seed_path: &str,
    ) -> Result<(), AppError> {
        // 1. Map business_type → file
        let file_name: &str = match business_type {
            "manufacturing" => "coa_manufacturing.json",
            "hospitality" => "coa_hospitality.json",
            "education" => "coa_education.json",
            "finance" => "coa_finance.json",
            "retail" => "coa_retail.json",
            "infotech" => "coa_infotechnology.json",
            "wholesome" => "coa_wholesale.json",
            "non_profit" => "coa_not_for_profit.json",
            "agriculture" => "coa_agriculture.json",
            "construction" => "coa_construction.json",
            "healthcare" => "coa_healthcare.json",
            "logistics" => "coa_logistics.json",
            "services" => "coa_professional_services.json",
            "energy" => "coa_energy.json",
            _ => {
                eprintln!("Invalid business_type received: {business_type}");
                return Err(AppError::BadRequest(format!(
                    "Invalid business_type: {business_type}"
                )));
            }
        };

        let path: String = format!("{coa_seed_path}/{file_name}");
        println!("\nReading COA file: {path}");

        // 2. Read file
        let contents: String = std::fs::read_to_string(&path).map_err(|e| {
            eprintln!("File not found or unreadable: {e}");
            AppError::Internal(format!("COA seed file missing: {path}"))
        })?;

        // 3. Parse the full template (this is the correct one now)
        let template: CoaTemplate = serde_json::from_str(&contents).map_err(|e| {
            eprintln!("JSON parsing failed: {e}");
            eprintln!(
                "First 500 chars of file:\n{}\n",
                &contents[..contents.len().min(500)]
            );
            AppError::Internal(format!("Invalid COA template in {file_name}: {e}"))
        })?;

        println!(
            "Loaded COA template: \"{}\" — {} accounts",
            template.name,
            template.accounts.len()
        );

        // 4. Insert each account from template.accounts
        for (counter, entry) in template.accounts.iter().enumerate() {
            let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query(
                r#"
            INSERT INTO accounting.accounts (
                code, name, category, parent_code, normal_balance, is_contra
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (code) DO NOTHING
            "#,
            )
            .bind(&entry.code)
            .bind(&entry.name)
            .bind(&entry.category)
            .bind(&entry.parent_code)
            .bind(&entry.normal_balance)
            .bind(entry.is_contra)
            .execute(tenant_pool)
            .await;

            match result {
                Ok(_) => {
                    if (counter + 1) % 50 == 0 || counter == template.accounts.len() - 1 {
                        println!("→ Inserted {} accounts so far...", counter + 1);
                    }
                }
                Err(e) => {
                    eprintln!("\nDATABASE INSERT FAILED at entry #{counter}");
                    eprintln!("Code: {}", entry.code);
                    eprintln!("Name: {}", entry.name);
                    eprintln!("Category: {}", entry.category);
                    eprintln!("Parent code: {:?}", entry.parent_code);
                    eprintln!("Error: {e}");
                    eprintln!("Full sqlx error: {e:?}\n");
                    return Err(AppError::Internal(format!(
                        "Failed to seed COA at entry {counter} (code: {}): {e}",
                        entry.code
                    )));
                }
            }
        }

        println!(
            "COA seeding completed successfully!\n→ Template: \"{}\"\n→ Industry: {}\n→ {} accounts inserted.\n",
            template.name,
            template.industry,
            template.accounts.len()
        );

        Ok(())
    }
}
