use crate::{
    features::capital::repository::CapitalRepository,
    interface::api::errors::AppError,
    models::capital::{
        CapitalFacility, CreateCapitalFacility, CreateDrawdown, CreateRepayment, DebtDrawdown,
        DebtRepayment,
    },
};

use sqlx::PgPool;
use uuid::Uuid;

pub struct CapitalService<R: CapitalRepository> {
    repo: R,
}

impl<R: CapitalRepository> CapitalService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // ---------------------------------------------------------
    // Debt Facilities
    // ---------------------------------------------------------

    pub async fn create_facility(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCapitalFacility,
    ) -> Result<CapitalFacility, AppError> {
        let mut tx = tenant_pool.begin().await?;

        let facility = self
            .repo
            .create_facility(tenant_pool, user_id, payload, &mut tx)
            .await?;

        tx.commit().await?;

        Ok(facility)
    }

    pub async fn get_facility(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError> {
        self.repo.get_facility(tenant_pool, id).await
    }

    pub async fn get_facilities(
        &self,
        tenant_pool: &PgPool,
    ) -> Result<Vec<CapitalFacility>, AppError> {
        self.repo.get_all_facilities(tenant_pool).await
    }

    // ---------------------------------------------------------
    // Drawdowns
    // ---------------------------------------------------------

    pub async fn create_drawdown(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateDrawdown,
    ) -> Result<DebtDrawdown, AppError> {
        let mut tx = tenant_pool.begin().await?;

        let drawdown = self
            .repo
            .create_drawdown(tenant_pool, user_id, payload, &mut tx)
            .await?;

        tx.commit().await?;

        Ok(drawdown)
    }

    pub async fn get_drawdowns(
        &self,
        tenant_pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError> {
        self.repo.get_drawdowns(tenant_pool, facility_id).await
    }

    // ---------------------------------------------------------
    // Repayments
    // ---------------------------------------------------------

    pub async fn create_repayment(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateRepayment,
    ) -> Result<DebtRepayment, AppError> {
        let mut tx = tenant_pool.begin().await?;

        let repayment = self
            .repo
            .create_repayment(tenant_pool, user_id, payload, &mut tx)
            .await?;

        tx.commit().await?;

        Ok(repayment)
    }

    pub async fn get_repayments(
        &self,
        tenant_pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError> {
        self.repo.get_repayments(tenant_pool, facility_id).await
    }
}
