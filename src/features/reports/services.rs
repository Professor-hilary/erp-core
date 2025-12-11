use crate::{features::reports::repository::ReportRepository, infrastructure::errors::AppError};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReportService<R: ReportRepository> {
    repo: R,
}

impl<R: ReportRepository> ReportService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn balancesheet(&self, tenant_pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        self.repo.get_balancesheet(tenant_pool, user_id).await
    }
    pub async fn income_statement(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_income(tenant_pool, user_id).await
    }
    pub async fn cf_direct(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_cf_direct(tenant_pool, uuid, user_id).await
    }
    pub async fn cf_indirect(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_cf_indirect(tenant_pool, uuid, user_id).await
    }
    pub async fn aging_ar(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_aging_ar(tenant_pool, uuid, user_id).await
    }

    pub async fn aging_ap(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_aging_ap(tenant_pool, uuid, user_id).await
    }

    pub async fn purchase_report(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .get_purchase_report(tenant_pool, uuid, user_id)
            .await
    }

    pub async fn sales_report(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.get_sales_report(tenant_pool, uuid, user_id).await
    }
}
