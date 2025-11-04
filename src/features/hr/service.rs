// src/features/hr/service.rs
use crate::{errors::AppError, features::hr::repository::HrRepository, models::{employee::{CreateEmployee, Employee}, payrun::{CreatePayrun, Payrun}}};
use sqlx::{PgPool};
use uuid::Uuid;

pub struct HrService<R: HrRepository> {
    repo: R,
}

impl<R: HrRepository> HrService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_employee(&self, user_id: Uuid, payload: &CreateEmployee) -> Result<Employee, AppError> {
        self.repo.create_employee(user_id, payload).await
    }

    pub async fn list_employees(&self, user_id: Uuid) -> Result<Vec<Employee>, AppError> {
        self.repo.list_employees(user_id).await
    }

    pub async fn get_employee(&self, id: i32, user_id: Uuid) -> Result<Employee, AppError> {
        self.repo.get_employee(id, user_id).await
    }

    pub async fn update_employee(&self, id: i32, user_id: Uuid, payload: &CreateEmployee) -> Result<Employee, AppError> {
        self.repo.update_employee(id, user_id, payload).await
    }

    pub async fn delete_employee(&self, id: i32, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete_employee(id, user_id).await
    }

    pub async fn create_and_post_payrun(&self, user_id: Uuid, payload: &CreatePayrun, pool: &PgPool) -> Result<Payrun, AppError> {
        let mut tx = pool.begin().await?;
        let payrun = self.repo.create_payrun(user_id, payload, &mut tx).await?;
        self.repo.create_payslips(payrun.payrun_id, &payload.payslips, &mut tx).await?;
        tx.commit().await?;
        self.repo.post_payrun(payrun.payrun_id).await?;
        Ok(payrun)
    }

    pub async fn get_payrun(&self, id: i32, user_id: Uuid) -> Result<Payrun, AppError> {
        self.repo.get_payrun(id, user_id).await
    }
}