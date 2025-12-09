// src/features/hr/service.rs
use crate::models::employee::{
    CreateDepartment, CreateEmployee, CreateJobTitle, Department, Employee, JobTitle,
};
use crate::{
    features::hr::repository::HrRepository,
    infrastructure::errors::AppError,
    models::payrun::{CreatePayrun, Payrun},
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

    pub async fn create_department(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        self.repo
            .create_department(tenant_pool, user_id, payload)
            .await
    }

    pub async fn create_job_title(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        self.repo
            .create_job_title(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_employees(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Employee>, AppError> {
        self.repo.list_employees(tenant_pool, user_id).await
    }

    pub async fn list_departments(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Department>, AppError> {
        self.repo.list_departments(tenant_pool, user_id).await
    }
    pub async fn list_job_titles(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JobTitle>, AppError> {
        self.repo.list_job_titles(tenant_pool, user_id).await
    }

    pub async fn get_employee(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        self.repo.get_employee(tenant_pool, id, user_id).await
    }

    pub async fn get_department(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Department, AppError> {
        self.repo.get_department(tenant_pool, id, user_id).await
    }
    pub async fn get_job_title(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<JobTitle, AppError> {
        self.repo.get_job_title(tenant_pool, id, user_id).await
    }

    pub async fn update_employee(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        self.repo
            .update_employee(tenant_pool, id, user_id, payload)
            .await
    }
    pub async fn update_department(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        self.repo
            .update_department(tenant_pool, id, user_id, payload)
            .await
    }
    pub async fn update_job_title(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        self.repo
            .update_job_title(tenant_pool, id, user_id, payload)
            .await
    }

    pub async fn delete_employee(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_employee(tenant_pool, id, user_id).await
    }

    pub async fn delete_department(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_department(tenant_pool, id, user_id).await
    }
    pub async fn delete_job_title(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_job_title(tenant_pool, id, user_id).await
    }

    pub async fn create_and_post_payrun(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePayrun,
    ) -> Result<Payrun, AppError> {
        let mut tx = tenant_pool.begin().await?;
        let payrun = self
            .repo
            .create_payrun(tenant_pool, user_id, payload, &mut tx)
            .await?;
        self.repo
            .create_payslips(tenant_pool, payrun.uuid, &payload.payslips, &mut tx)
            .await?;
        tx.commit().await?;
        self.repo.post_payrun(tenant_pool, payrun.uuid).await?;
        Ok(payrun)
    }

    pub async fn get_payrun(
        &self,
        tenant_pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Payrun, AppError> {
        self.repo.get_payrun(tenant_pool, id, user_id).await
    }
}
