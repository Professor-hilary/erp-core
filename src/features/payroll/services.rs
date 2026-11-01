// src/features/payroll/services.rs
use crate::{
    features::payroll::repository::PayrollRepository,
    interface::api::errors::AppError,
    models::payroll::*,
};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
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
                .map(|employee| CreatePayslip {
                    employee_uuid: employee.employee_uuid,
                    gross_pay: BigDecimal::from(0),
                    basic_pay: Some(BigDecimal::from(0)),
                    tax_deducted: Some(BigDecimal::from(0)),
                    social_security: Some(BigDecimal::from(0)),
                    employer_social_security: Some(BigDecimal::from(0)),
                    other_deductions: Some(BigDecimal::from(0)),
                    payment_method: employee.payment_method,
                    bank_account: employee.bank_account_number,
                    benefits: None,
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

    // --- Component assignments ---

    pub async fn create_component_assignment(
        &self,
        pool: &PgPool,
        payload: &CreateComponentAssignment,
    ) -> Result<EmployeeComponentAssignment, AppError> {
        self.repo.create_component_assignment(pool, payload).await
    }

    pub async fn list_component_assignments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmployeeComponentAssignment>, AppError> {
        self.repo
            .list_component_assignments(pool, employee_uuid)
            .await
    }

    // --- Payslip breakdown ---

    pub async fn get_payslip_calculation_items(
        &self,
        pool: &PgPool,
        payslip_uuid: Uuid,
    ) -> Result<Vec<PayslipCalculationItem>, AppError> {
        self.repo
            .get_payslip_calculation_items(pool, payslip_uuid)
            .await
    }

    // --- Settlement ---

    pub async fn settle_employees(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &SettleEmployees,
    ) -> Result<EmployeePaymentBatch, AppError> {
        self.repo.settle_employees(user_id, pool, payload).await
    }

    pub async fn get_payment_batch(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Option<EmployeePaymentBatch>, AppError> {
        self.repo
            .get_payment_batch_for_payrun(pool, payrun_uuid)
            .await
    }

    // --- Statutory remittance ---

    pub async fn create_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
        due_date: NaiveDate,
    ) -> Result<Vec<StatutoryRemittance>, AppError> {
        self.repo
            .create_statutory_remittances(pool, payrun_uuid, due_date)
            .await
    }

    pub async fn remit_statutory(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &RemitStatutory,
    ) -> Result<StatutoryRemittance, AppError> {
        self.repo.remit_statutory(user_id, pool, payload).await
    }

    pub async fn list_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Option<Uuid>,
    ) -> Result<Vec<StatutoryRemittance>, AppError> {
        self.repo
            .list_statutory_remittances(pool, payrun_uuid)
            .await
    }
}
