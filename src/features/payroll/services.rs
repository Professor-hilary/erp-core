// src/features/payroll/services.rs
use crate::{
    features::payroll::repository::PayrollRepository,
    interface::api::errors::AppError,
    models::payrun::{
        CreatePayComponent, CreatePayslip, CreatePayrun, PayComponent, Payrun, Payslip, PostPayrun,
    },
};
use bigdecimal::BigDecimal;
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
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreatePayrun,
    ) -> Result<Payrun, AppError> {
        let mut tx = pool.begin().await?;
        let payrun = self.repo.create_payrun(payload, &mut tx).await?;

        let slips: Vec<CreatePayslip> = if payload.payslips.is_empty() {
            let eligible = self
                .repo
                .list_eligible_employees(pool, payload.pay_period_start, payload.pay_period_end)
                .await?;
            if eligible.is_empty() {
                return Err(AppError::Unprocessable(
                    "No payroll-eligible employees for this period (check HR status/contracts)"
                        .into(),
                ));
            }
            eligible
                .into_iter()
                .map(|e| {
                    let basic = e
                        .basic_salary
                        .clone()
                        .unwrap_or_else(|| BigDecimal::from(0));
                    CreatePayslip {
                        employee_uuid: e.employee_uuid,
                        gross_pay: basic.clone(),
                        basic_pay: Some(basic),
                        tax_deducted: Some(BigDecimal::from(0)),
                        social_security: Some(BigDecimal::from(0)),
                        employer_social_security: Some(BigDecimal::from(0)),
                        other_deductions: Some(BigDecimal::from(0)),
                        payment_method: e.payment_method,
                        bank_account: e.bank_account_number,
                        benefits: None,
                    }
                })
                .collect()
        } else {
            payload.payslips.clone()
        };

        self.repo
            .create_payslips(payrun.uuid, &slips, &mut tx)
            .await?;
        tx.commit().await?;
        Ok(payrun)
    }

    pub async fn process_payrun(
        &self,
        pool: &PgPool,
        payrun_serial_id: i64,
    ) -> Result<Payrun, AppError> {
        self.repo.process_payrun(pool, payrun_serial_id).await
    }

    pub async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError> {
        self.repo.post_payrun(user_id, pool, payload).await
    }

    pub async fn get_payrun(&self, pool: &PgPool, id: Uuid) -> Result<Payrun, AppError> {
        self.repo.get_payrun(pool, id).await
    }

    pub async fn get_payruns(&self, pool: &PgPool) -> Result<Vec<Payrun>, AppError> {
        self.repo.get_all_payruns(pool).await
    }

    pub async fn get_payslips(&self, pool: &PgPool) -> Result<Vec<Payslip>, AppError> {
        self.repo.get_all_payslips(pool).await
    }

    pub async fn get_payslips_for_payrun(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        self.repo.get_payslips_for_payrun(pool, payrun_uuid).await
    }

    pub async fn create_pay_component(
        &self,
        pool: &PgPool,
        payload: &CreatePayComponent,
    ) -> Result<PayComponent, AppError> {
        self.repo.create_pay_component(pool, payload).await
    }

    pub async fn list_pay_components(&self, pool: &PgPool) -> Result<Vec<PayComponent>, AppError> {
        self.repo.list_pay_components(pool).await
    }
}

/*
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
*/