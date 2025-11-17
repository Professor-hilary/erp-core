// src/features/hr/service.rs
use crate::{
    errors::AppError,
    features::hr::repository::HrRepository,
    models::{
        employee::{CreateEmployee, Employee},
        payrun::{CreatePayrun, Payrun},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct HrService<R: HrRepository> {
    repo: R,
}

impl<R: HrRepository> HrService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_employee(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        self.repo
            .create_employee(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_employees(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Employee>, AppError> {
        self.repo.list_employees(tenant_pool, user_id).await
    }

    pub async fn get_employee(
        &self,
        tenant_pool: &PgPool,
        id: i32,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        self.repo.get_employee(tenant_pool, id, user_id).await
    }

    pub async fn update_employee(
        &self,
        tenant_pool: &PgPool,
        id: i32,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        self.repo
            .update_employee(tenant_pool, id, user_id, payload)
            .await
    }

    pub async fn delete_employee(
        &self,
        tenant_pool: &PgPool,
        id: i32,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_employee(tenant_pool, id, user_id).await
    }

    pub async fn create_and_post_payrun(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePayrun,
        // pool: &PgPool,
    ) -> Result<Payrun, AppError> {
        let mut tx = tenant_pool.begin().await?;
        let payrun = self
            .repo
            .create_payrun(tenant_pool, user_id, payload, &mut tx)
            .await?;
        self.repo
            .create_payslips(tenant_pool, payrun.payrun_id, &payload.payslips, &mut tx)
            .await?;
        tx.commit().await?;
        self.repo.post_payrun(tenant_pool, payrun.payrun_id).await?;
        Ok(payrun)
    }

    pub async fn get_payrun(
        &self,
        tenant_pool: &PgPool,
        id: i32,
        user_id: Uuid,
    ) -> Result<Payrun, AppError> {
        self.repo.get_payrun(tenant_pool, id, user_id).await
    }
}
