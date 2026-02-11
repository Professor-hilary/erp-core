// src/features/payroll/service.rs
use crate::{
    features::payroll::repository::PayrollRepository,
    interface::api::errors::AppError,
    models::payrun::{CreatePayrun, Payrun, Payslip, PostPayrun},
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PayrollService<R: PayrollRepository> {
    repo: R,
}

impl<R: PayrollRepository> PayrollService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
    pub async fn create_payrun(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePayrun,
    ) -> Result<Payrun, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = tenant_pool.begin().await?;
        let payrun: Payrun = self
            .repo
            .create_payrun(tenant_pool, user_id, payload, &mut tx)
            .await?;
        self.repo
            .create_payslips(tenant_pool, payrun.uuid, &payload.payslips, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(payrun)
    }

    pub async fn process_payrun(
        &self,
        tenant_pool: &PgPool,
        payrun_id: i64,
    ) -> Result<Payrun, AppError> {
        self.repo.process_payrun(tenant_pool, payrun_id).await
    }

    pub async fn post_payrun(
        &self,
        user_id: Uuid,
        tenant_pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError> {
        self.repo.post_payrun(user_id, tenant_pool, payload).await?;
        Ok(())
    }

    pub async fn get_payrun(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Payrun, AppError> {
        self.repo.get_payrun(tenant_pool, id, user_id).await
    }

    pub async fn get_payruns(&self, tenant_pool: &PgPool) -> Result<Vec<Payrun>, AppError> {
        self.repo.get_all_payruns(tenant_pool).await
    }

    pub async fn get_payslips(&self, tenant_pool: &PgPool) -> Result<Vec<Payslip>, AppError> {
        self.repo.get_all_payslips(tenant_pool).await
    }
}
