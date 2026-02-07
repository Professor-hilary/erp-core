// src/features/company/repository.rs
use crate::infrastructure::errors::AppError;
use crate::models::company::{Company, UpdateCompanyDto};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

#[async_trait]
pub trait CompanyRepository: Send + Sync {
    /// # Create Company
    /// Create a new company for currently signed user
    async fn create<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        name: &str,
        slug: &str,
        tenant_db_name: &str,
        tenant_db_uri: &str,
        industry: &str,
        currency: &str,
        business_type: &str,
        country: &str,
        company_email: Option<String>,
        legal_name: Option<String>,
        telephone: Option<String>,
        website: Option<String>,
        co_address: Option<String>,
        city: Option<String>,
        co_state: Option<String>,
        zip_code: Option<String>,
        tax_id: Option<String>,
        period_start: Option<NaiveDate>,
        period_type: Option<String>,
        created_by: Uuid,
    ) -> Result<Company, AppError>;

    /// # Assign Admin Privilege
    /// Make current user the admin for created company
    async fn assign_user_as_admin<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError>;

    /// # Assign Admin Privilege
    /// Make current user the admin for created company
    async fn update_company_secret<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
        tenant_db_uri: &str,
        company_id: Uuid,
        tenant_db_name: String,
        tenant_db_role: String,
        tenant_db_pass: String,
        tenant_db_host: String,
        tenant_db_port: i32,
    ) -> Result<(), AppError>;

    /// # Find By Id
    /// Find company by id provided in master database
    async fn find_by_id<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
    ) -> Result<Option<Company>, AppError>;

    /// # Find By User
    /// Find all companies registered to current signed user
    async fn find_by_user<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
    ) -> Result<Vec<Company>, AppError>;

    /// # Update Company Details
    /// Update current company details, user must be the admin
    async fn update<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
        dto: UpdateCompanyDto,
    ) -> Result<Company, AppError>;

    /// # Soft Delete Company
    /// Mark company as deleted for audit and recovery purpose
    async fn soft_delete<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn update_status<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
        status: &str,
    ) -> Result<(), AppError>;
}

pub struct PostgresCompanyRepository;

#[async_trait]
impl CompanyRepository for PostgresCompanyRepository {
    async fn create<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        name: &str,
        slug: &str,
        tenant_db_name: &str,
        tenant_db_uri: &str,
        industry: &str,
        currency: &str,
        business_type: &str,
        country: &str,
        company_email: Option<String>,
        legal_name: Option<String>,
        telephone: Option<String>,
        website: Option<String>,
        co_address: Option<String>,
        city: Option<String>,
        co_state: Option<String>,
        zip_code: Option<String>,
        tax_id: Option<String>,
        period_start: Option<NaiveDate>,
        period_type: Option<String>,
        created_by: Uuid,
    ) -> Result<Company, AppError> {
        let company: Company = sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (
                name, slug, tenant_db_name, tenant_db_uri,
                industry, business_type, created_by,
                country, currency, company_email, legal_name,
                telephone, website, co_address, city, co_state,
                zip_code, tax_id, period_start, period_type
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16, $17, $18, $19, $20
            ) RETURNING *
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(tenant_db_name)
        .bind(tenant_db_uri)
        .bind(industry)
        .bind(business_type)
        .bind(created_by)
        .bind(country)
        .bind(currency)
        .bind(company_email)
        .bind(legal_name)
        .bind(telephone)
        .bind(website)
        .bind(co_address)
        .bind(city)
        .bind(co_state)
        .bind(zip_code)
        .bind(tax_id)
        .bind(period_start)
        .bind(period_type)
        .fetch_one(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(company)
    }

    async fn assign_user_as_admin<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_companies (user_id, company_id, role)
            VALUES ($1, $2, 'admin')
            ON CONFLICT (user_id, company_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(company_id)
        .execute(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn update_company_secret<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
        tenant_db_uri: &str,
        company_id: Uuid,
        tenant_db_name: String,
        tenant_db_role: String,
        tenant_db_pass: String,
        tenant_db_host: String,
        tenant_db_port: i32,
    ) -> Result<(), AppError> {
        // assume `company_id` is known and `tenant_url` is built
        sqlx::query(
            r#"
                INSERT INTO tenant_secrets (
                    company_id,
                    secret,
                    db_name,
                    db_host,
                    db_port,
                    db_user,
                    db_password,
                    created_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (company_id) DO NOTHING"#,
        )
        .bind(company_id)
        .bind(serde_json::json!({ "tenant_url": tenant_db_uri }))
        .bind(tenant_db_name)
        .bind(tenant_db_host)
        .bind(tenant_db_port)
        .bind(tenant_db_role)
        .bind(tenant_db_pass)
        .bind(user_id)
        .execute(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
    ) -> Result<Option<Company>, AppError> {
        let company = sqlx::query_as::<_, Company>(
            "SELECT * FROM companies WHERE uuid = $1 AND status != 'deleted'",
        )
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(company)
    }

    async fn find_by_user<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        user_id: Uuid,
    ) -> Result<Vec<Company>, AppError> {
        let companies = sqlx::query_as::<_, Company>(
            r#"
            SELECT c.* FROM companies c
            JOIN user_companies uc ON c.uuid = uc.company_id
            WHERE uc.user_id = $1 AND c.status != 'deleted'
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(companies)
    }

    async fn update<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
        dto: UpdateCompanyDto,
    ) -> Result<Company, AppError> {
        let new_slug = dto
            .name
            .as_ref()
            .map(|n| n.to_lowercase().replace(" ", "-"));

        let company: Company = sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET
                name = COALESCE($1, name),
                slug = COALESCE($2, slug),
                industry = COALESCE($3, industry),
                business_type = COALESCE($4, business_type)
                country = COALESCE($5, country)
                currency = COALESCE($5, currency)
                company_email = COALESCE($6, company_email)
                legal_name = COALESCE($7, legal_name)
                telephone = COALESCE($8, telephone)
                website = COALESCE($9, website)
                co_address = COALESCE($10, co_address)
                city = COALESCE($11, city)
                co_state = COALESCE($12, co_state)
                zip_code = COALESCE($13, zip_code)
                tax_id = COALESCE($14, tax_id)
                period_start = COALESCE($15, period_start)
                period_end = COALESCE($15, period_end)
            WHERE uuid = $16 AND status != 'deleted'
            RETURNING *
            "#,
        )
        .bind(dto.name.as_ref())
        .bind(new_slug.as_ref())
        .bind(dto.industry.as_ref())
        .bind(dto.business_type.as_ref())
        .bind(dto.country.as_ref())
        .bind(dto.currency.as_ref())
        .bind(dto.company_email.as_ref())
        .bind(dto.legal_name.as_ref())
        .bind(dto.telephone.as_ref())
        .bind(dto.website.as_ref())
        .bind(dto.co_address.as_ref())
        .bind(dto.city.as_ref())
        .bind(dto.co_state.as_ref())
        .bind(dto.zip_code.as_ref())
        .bind(dto.tax_id.as_ref())
        .bind(dto.period_start.as_ref())
        .bind(dto.period_end.as_ref())
        .bind(id)
        .fetch_one(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(company)
    }

    async fn soft_delete<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE companies SET status = 'deleted' WHERE uuid = $1 AND status != 'deleted'",
        )
        .bind(id)
        .execute(executor)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update_status<'e, E: Executor<'e, Database = Postgres>>(
        &self,
        executor: E,
        id: Uuid,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE companies SET status=$1 WHERE uuid=$2")
            .bind(status)
            .bind(id)
            .execute(executor)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}
