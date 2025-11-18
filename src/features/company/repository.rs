// src/features/company/repository.rs
use crate::{
    errors::AppError,
    models::company::{Company, UpdateCompanyDto},
};
use async_trait::async_trait;
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
        business_type: &str,
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
        business_type: &str,
        created_by: Uuid,
    ) -> Result<Company, AppError> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (
                name, slug, tenant_db_name, tenant_db_uri,
                industry, business_type, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(slug)
        .bind(tenant_db_name)
        .bind(tenant_db_uri)
        .bind(industry)
        .bind(business_type)
        .bind(created_by)
        .fetch_one(executor)
        .await?;

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
        .await?;
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
        .await?;
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
        .await?;
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

        let company = sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET
                name = COALESCE($1, name),
                slug = COALESCE($2, slug),
                industry = COALESCE($3, industry),
                business_type = COALESCE($4, business_type)
            WHERE uuid = $5 AND status != 'deleted'
            RETURNING *
            "#,
        )
        .bind(dto.name.as_ref())
        .bind(new_slug.as_ref())
        .bind(dto.industry.as_ref())
        .bind(dto.business_type.as_ref())
        .bind(id)
        .fetch_one(executor)
        .await?;

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
        .await?;
        Ok(())
    }
}
