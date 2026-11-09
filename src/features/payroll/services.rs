// src/features/payroll/services.rs
use crate::{
    features::accounts::account_roles::{AccountRole, AccountRoleResolver},
    features::payroll::repository::PayrollRepository,
    interface::api::errors::AppError,
    models::payroll::*,
};
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

fn role_uuid(
    resolver: &AccountRoleResolver,
    role: AccountRole,
    override_id: Option<Uuid>,
) -> Result<Uuid, AppError> {
    if let Some(id) = override_id {
        return Ok(id);
    }
    resolver.resolve_default(role).map(|a| a.uuid).map_err(|e| {
        AppError::Unprocessable(format!(
            "Account role '{}' is not mapped: {e}",
            role.key()
        ))
    })
}

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
                .map(|employee| {
					let basic = employee
				        .basic_salary
				        .clone()
				        .unwrap_or_else(|| BigDecimal::from(0));
					CreatePayslip {
	                    employee_uuid: employee.employee_uuid,
	                    gross_pay: BigDecimal::from(0),
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
        let resolver = AccountRoleResolver::load(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to load account roles: {e}")))?;
        let resolved = PostPayrun {
            payrun_serial_id: payload.payrun_serial_id,
            labor_expense_id: Some(role_uuid(&resolver, AccountRole::PayrollLabourExpense, payload.labor_expense_id)?),
            income_tax_id: Some(role_uuid(&resolver, AccountRole::PayePayable, payload.income_tax_id)?),
            employee_social_security_id: Some(role_uuid(&resolver, AccountRole::EmployeeNssfPayable, payload.employee_social_security_id)?),
            employer_social_security_id: Some(role_uuid(&resolver, AccountRole::EmployerNssfPayable, payload.employer_social_security_id)?),
            employer_social_security_expense_id: Some(role_uuid(&resolver, AccountRole::EmployerNssfExpense, payload.employer_social_security_expense_id)?),
            payroll_payable: Some(role_uuid(&resolver, AccountRole::PayrollNetPayable, payload.payroll_payable)?),
            other_deductions_payable_id: payload.other_deductions_payable_id.or_else(|| {
                resolver.resolve_default(AccountRole::OtherPayrollDeductionsPayable).ok().map(|a| a.uuid)
            }),
            other_employer_cost_expense_id: payload.other_employer_cost_expense_id.or_else(|| {
                resolver.resolve_default(AccountRole::OtherEmployerPayrollExpense).ok().map(|a| a.uuid)
            }),
        };
        self.repo.post_payrun(user_id, pool, &resolved).await
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

    pub async fn list_pay_components(
		&self, 
		pool: &PgPool
	) -> Result<Vec<PayComponent>, AppError> {
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
        let resolver = AccountRoleResolver::load(pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to load account roles: {e}")))?;
        let cash = if let Some(id) = payload.cash_account_uuid {
            id
        } else if let Ok(a) = resolver.resolve_default(AccountRole::CashAtBank) {
            a.uuid
        } else {
            role_uuid(&resolver, AccountRole::CashAtHand, None)?
        };
        let payable = role_uuid(&resolver, AccountRole::PayrollNetPayable, payload.payroll_payable_uuid)?;
        let resolved = SettleEmployees {
            payrun_uuid: payload.payrun_uuid,
            payment_date: payload.payment_date,
            payment_reference: payload.payment_reference.clone(),
            cash_account_uuid: Some(cash),
            payroll_payable_uuid: Some(payable),
        };
        self.repo.settle_employees(user_id, pool, &resolved).await
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
		remittance: &CreateStatutoryRemittance,
        // payrun_uuid: Uuid,
        // due_date: NaiveDate,
    ) -> Result<Vec<StatutoryRemittance>, AppError> {
        self.repo
            .create_statutory_remittances(pool, remittance/*payrun_uuid, due_date*/)
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
