// src/features/payroll/services.rs
use crate::{
    features::payroll::repository::PayrollRepository, interface::api::errors::AppError,
    models::payroll::*,
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
        if payload.pay_period_start > payload.pay_period_end {
            return Err(AppError::Unprocessable(
                "Pay period end cannot precede its start".into(),
            ));
        }

        if payload.payment_date < payload.pay_period_end {
            return Err(AppError::Unprocessable(
                "Payment date cannot precede the end of the pay period".into(),
            ));
        }

        let mut employees = std::collections::HashSet::new();

        for slip in &payload.payslips {
            if !employees.insert(slip.employee_uuid) {
                return Err(AppError::Unprocessable(
                    "Duplicate employee in payrun".into(),
                ));
            }

            let zero = BigDecimal::from(0);

            if slip.gross_pay < zero
                || slip.basic_pay.as_ref().is_some_and(|v| v < &zero)
                || slip.tax_deducted.as_ref().is_some_and(|v| v < &zero)
                || slip.social_security.as_ref().is_some_and(|v| v < &zero)
                || slip
                    .employer_social_security
                    .as_ref()
                    .is_some_and(|v| v < &zero)
                || slip.other_deductions.as_ref().is_some_and(|v| v < &zero)
            {
                return Err(AppError::Unprocessable(
                    "Payroll amounts cannot be negative".into(),
                ));
            }

            if let Some(items) = &slip.benefits {
                for item in items {
                    if !["Allowance", "Deduction"].contains(&item.item_type.as_str()) {
                        return Err(AppError::Unprocessable(
                            "Payslip item must be Allowance or Deduction".into(),
                        ));
                    }

                    if item.amount < zero {
                        return Err(AppError::Unprocessable(
                            "Payslip item amount cannot be negative".into(),
                        ));
                    }
                }
            }
        }

        let mut tx = pool.begin().await?;

        let payrun = self.repo.create_payrun(payload, &mut tx).await?;

        let slips: Vec<CreatePayslip> = if payload.payslips.is_empty() {
            let eligible = self
                .repo
                .list_eligible_employees(pool, payload.pay_period_start, payload.pay_period_end)
                .await?;

            if eligible.is_empty() {
                return Err(AppError::Unprocessable(
                    "No payroll-eligible employees for this period".into(),
                ));
            }

            eligible
                .into_iter()
                .map(|employee| {
                    let basic = employee
                        .basic_salary
                        .clone()
                        .unwrap_or_else(|| BigDecimal::from(0));

                    CreatePayslip {
                        employee_uuid: employee.employee_uuid,
                        gross_pay: basic.clone(),
                        basic_pay: Some(basic),
                        tax_deducted: Some(BigDecimal::from(0)),
                        social_security: Some(BigDecimal::from(0)),
                        employer_social_security: Some(BigDecimal::from(0)),
                        other_deductions: Some(BigDecimal::from(0)),
                        payment_method: employee.payment_method,
                        bank_account: employee.bank_account_number,
                        benefits: None,
                    }
                })
                .collect()
        } else {
            payload.payslips.clone()
        };

        let mut generated_employees = std::collections::HashSet::new();

        for slip in &slips {
            if !generated_employees.insert(slip.employee_uuid) {
                return Err(AppError::Unprocessable(
                    "Duplicate employee in generated payrun".into(),
                ));
            }
        }

        self.repo
            .create_payslips(payrun.uuid, &slips, &mut tx)
            .await?;

        tx.commit().await?;

        Ok(payrun)
    }

    /*pub async fn create_payrun(
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
    }*/

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
