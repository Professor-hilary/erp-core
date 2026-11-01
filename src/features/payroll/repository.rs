// src/features/payroll/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::payroll::{
        CreatePayComponent, CreatePayrun, CreatePayslip, EligibleEmployeeRow, PayComponent, Payrun,
        Payslip, PostPayrun,
    },
};
use async_trait::async_trait;
// use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/*#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentKind {
    Earning,
    Deduction,
    EmployerCost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalculationMethod {
    Fixed,
    PercentOfBasic,
    PercentOfGross,
    Manual,
    External,
}
 
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayrollCalculationError {
    InvalidPeriod,
    InvalidContractPeriod,
    NegativeBasicSalary,
    MissingTaxBands,
    InvalidTaxResidency,
    InvalidComponentAmount(String),
    InvalidComponentRate(String),
    DuplicateComponent(String),
    NegativeNetPay,
}
*/
#[async_trait]
pub trait PayrollRepository: Send + Sync {
    async fn create_payrun(
        &self,
        payload: &CreatePayrun,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Payrun, AppError>;

    async fn create_payslips(
        &self,
        payrun_uuid: Uuid,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError>;

    async fn list_eligible_employees(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<EligibleEmployeeRow>, AppError>;

    async fn process_payrun(
        &self,
        pool: &PgPool,
        payrun_serial_id: i64,
    ) -> Result<Payrun, AppError>;

    async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError>;

    async fn get_payrun(&self, pool: &PgPool, id: Uuid) -> Result<Payrun, AppError>;
    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Vec<Payrun>, AppError>;
    async fn get_all_payslips(&self, pool: &PgPool) -> Result<Vec<Payslip>, AppError>;
    async fn get_payslips_for_payrun(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Vec<Payslip>, AppError>;

    async fn create_pay_component(
        &self,
        pool: &PgPool,
        payload: &CreatePayComponent,
    ) -> Result<PayComponent, AppError>;
    async fn list_pay_components(&self, pool: &PgPool) -> Result<Vec<PayComponent>, AppError>;
}

pub struct PostgresPayrollRepo;

impl PostgresPayrollRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PayrollRepository for PostgresPayrollRepo {
    async fn create_payrun(
        &self,
        payload: &CreatePayrun,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Payrun, AppError> {
        let payrun = sqlx::query_as::<_, Payrun>(
            r#"
            INSERT INTO payroll.payruns (pay_period_start, pay_period_end, payment_date, notes)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(payload.pay_period_start)
        .bind(payload.pay_period_end)
        .bind(payload.payment_date)
        .bind(&payload.notes)
        .fetch_one(&mut **tx)
        .await?;
        Ok(payrun)
    }

    async fn create_payslips(
        &self,
        payrun_uuid: Uuid,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError> {
        let mut employees = std::collections::HashSet::new();

        for slip in payslips {
            if !employees.insert(slip.employee_uuid) {
                return Err(AppError::Unprocessable(
                    "Duplicate employee in payrun".into(),
                ));
            }

            let tax = slip.tax_deducted.clone().unwrap_or_default();
            let ss = slip.social_security.clone().unwrap_or_default();
            let other = slip.other_deductions.clone().unwrap_or_default();
            let employer_ss = slip.employer_social_security.clone().unwrap_or_default();

            let method = slip
                .payment_method
                .clone()
                .unwrap_or_else(|| "Bank Transfer".into());

            let payslip_uuid: Uuid = sqlx::query_scalar(
                r#"
	            INSERT INTO payroll.payslips (
	                payrun_uuid,
	                employee_uuid,
	                gross_pay,
	                basic_pay,
	                tax_deducted,
	                social_security,
	                employer_social_security,
	                other_deductions,
	                payment_method,
	                bank_account
	            )
	            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
	            RETURNING uuid
	            "#,
            )
            .bind(payrun_uuid)
            .bind(slip.employee_uuid)
            .bind(&slip.gross_pay)
            .bind(&slip.basic_pay)
            .bind(&tax)
            .bind(&ss)
            .bind(&employer_ss)
            .bind(&other)
            .bind(&method)
            .bind(&slip.bank_account)
            .fetch_one(&mut **tx)
            .await?;

            if let Some(items) = &slip.benefits {
                for item in items {
                    if !["Allowance", "Deduction"].contains(&item.item_type.as_str()) {
                        return Err(AppError::Unprocessable("Invalid payslip item type".into()));
                    }

                    sqlx::query(
                        r#"
	                    INSERT INTO payroll.payslip_items (
	                        payslip_uuid,
	                        item_type,
	                        description,
	                        amount,
	                        component_uuid
	                    )
	                    VALUES ($1,$2,$3,$4,$5)
	                    "#,
                    )
                    .bind(payslip_uuid)
                    .bind(&item.item_type)
                    .bind(&item.description)
                    .bind(&item.amount)
                    .bind(item.component_uuid)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }

        Ok(())
    }

    async fn list_eligible_employees(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<EligibleEmployeeRow>, AppError> {
        // Uses HR function + active contract + payment profile
        let rows = sqlx::query_as::<_, EligibleEmployeeRow>(
            r#"
            SELECT
                e.uuid AS employee_uuid,
                c.basic_salary,
                pp.bank_account_number,
                pp.payment_method
            FROM hr.employees e
            LEFT JOIN hr.employment_contracts c
                ON c.employee_uuid = e.uuid AND c.status = 'Active'
            LEFT JOIN hr.employee_payment_profiles pp
                ON pp.employee_uuid = e.uuid
            WHERE hr.is_payroll_eligible(e.uuid, $1, $2)
            ORDER BY e.last_name, e.first_name
            "#,
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    async fn process_payrun(
        &self,
        pool: &PgPool,
        payrun_serial_id: i64,
    ) -> Result<Payrun, AppError> {
        let mut tx = pool.begin().await?;

		let uncalculated_count: i64 = sqlx::query_scalar(
		    r#"
		    SELECT COUNT(*)
		    FROM payroll.payslips
		    WHERE payrun_uuid = $1
		      AND (
		          calculated_at IS NULL
		          OR calculation_version IS NULL
		      )
		    "#,
		)
		.bind(payrun.uuid)
		.fetch_one(&mut *tx)
		.await?;

		if uncalculated_count > 0 {
		    return Err(AppError::Unprocessable(
		        format!(
		            "{} payslip(s) have not been calculated",
		            uncalculated_count
		        ),
		    ));
		}

        let payrun: Payrun = sqlx::query_as(
            r#"
	        SELECT *
	        FROM payroll.payruns
	        WHERE serial_id = $1
	        FOR UPDATE
	        "#,
        )
        .bind(payrun_serial_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Payrun not found".into()))?;

        if payrun.status != "Pending" {
            return Err(AppError::Unprocessable(format!(
                "Cannot process payrun with status {}",
                payrun.status
            )));
        }

        let slip_count: i64 = sqlx::query_scalar(
            r#"
	        SELECT COUNT(*)
	        FROM payroll.payslips
	        WHERE payrun_uuid = $1
	        "#,
        )
        .bind(payrun.uuid)
        .fetch_one(&mut *tx)
        .await?;

        if slip_count == 0 {
            return Err(AppError::Unprocessable(
                "Cannot process an empty payrun".into(),
            ));
        }

        let invalid_count: i64 = sqlx::query_scalar(
            r#"
	        SELECT COUNT(*)
	        FROM payroll.payslips
	        WHERE payrun_uuid = $1
	          AND (
	              gross_pay < 0
	              OR tax_deducted < 0
	              OR social_security < 0
	              OR other_deductions < 0
	              OR net_pay < 0
	          )
	        "#,
        )
        .bind(payrun.uuid)
        .fetch_one(&mut *tx)
        .await?;

        if invalid_count > 0 {
            return Err(AppError::Unprocessable(
                "Payrun contains invalid payslip amounts".into(),
            ));
        }

        let updated = sqlx::query_as::<_, Payrun>(
            r#"
	        UPDATE payroll.payruns
	        SET status = 'Processed',
	            processed_at = NOW(),
	            updated_at = NOW()
	        WHERE uuid = $1
	          AND status = 'Pending'
	        RETURNING *
	        "#,
        )
        .bind(payrun.uuid)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(updated)
    }

async fn post_payrun(
    &self,
    user_id: Uuid,
    pool: &PgPool,
    payload: &PostPayrun,
) -> Result<(), AppError> {
    let labor = payload.labor_expense_id.ok_or_else(|| {
        AppError::Unprocessable("Payroll expense account is required".into())
    })?;

    let tax = payload.income_tax_id.ok_or_else(|| {
        AppError::Unprocessable("PAYE payable account is required".into())
    })?;

    let employee_nssf = payload.employee_social_security_id.ok_or_else(|| {
        AppError::Unprocessable("Employee NSSF payable account is required".into())
    })?;

    let employer_nssf = payload.employer_social_security_id.ok_or_else(|| {
        AppError::Unprocessable("Employer NSSF payable account is required".into())
    })?;

    let employer_nssf_expense =
        payload.employer_social_security_expense_id.ok_or_else(|| {
            AppError::Unprocessable(
                "Employer NSSF expense account is required".into(),
            )
        })?;

    let payable = payload.payroll_payable.ok_or_else(|| {
        AppError::Unprocessable("Payroll payable account is required".into())
    })?;

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        SELECT payroll.post_payrun(
            $1,$2,$3,$4,$5,$6,$7,$8
        )
        "#,
    )
    .bind(payload.payrun_serial_id)
    .bind(labor)
    .bind(tax)
    .bind(employee_nssf)
    .bind(employer_nssf)
    .bind(employer_nssf_expense)
    .bind(payable)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

    /*async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError> {
        let labor = payload.labor_expense_id.ok_or_else(|| {
            AppError::Unprocessable(
                "labor_expense_id required until account roles resolve posts".into(),
            )
        })?;
        let tax = payload.income_tax_id.ok_or_else(|| {
            AppError::Unprocessable(
                "income_tax_id required until account roles resolve posts".into(),
            )
        })?;
        let ss = payload.social_security_id.ok_or_else(|| {
            AppError::Unprocessable(
                "social_security_id required until account roles resolve posts".into(),
            )
        })?;
        let cash = payload.cash_account_uuid.ok_or_else(|| {
            AppError::Unprocessable(
                "cash_account_uuid required until account roles resolve posts".into(),
            )
        })?;
        let payable = payload.payroll_payable.ok_or_else(|| {
            AppError::Unprocessable(
                "payroll_payable required until account roles resolve posts".into(),
            )
        })?;

        // Guard: only Processed
        let status: Option<(String,)> =
            sqlx::query_as("SELECT status FROM payroll.payruns WHERE serial_id = $1")
                .bind(payload.payrun_serial_id)
                .fetch_optional(pool)
                .await?;

        match status {
            Some((ref s,)) if s == "Processed" => {}
            Some((s,)) => {
                return Err(AppError::Unprocessable(format!(
                    "Payrun must be Processed to post (current: {s})"
                )));
            }
            None => return Err(AppError::NotFound("Payrun not found".into())),
        }

        sqlx::query("SELECT payroll.post_payrun($1, $2, $3, $4, $5, $6, $7)")
            .bind(payload.payrun_serial_id)
            .bind(labor)
            .bind(tax)
            .bind(ss)
            .bind(cash)
            .bind(user_id)
            .bind(payable)
            .execute(pool)
            .await?;

        sqlx::query(
            r#"
            UPDATE payroll.payruns
            SET status = 'Posted', posted_at = now(), updated_at = now()
            WHERE serial_id = $1 AND status = 'Processed'
            "#,
        )
        .bind(payload.payrun_serial_id)
        .execute(pool)
        .await?;

        Ok(())
    }*/

    async fn get_payrun(&self, pool: &PgPool, id: Uuid) -> Result<Payrun, AppError> {
        sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns WHERE uuid = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Payrun not found".into()))
    }

    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Vec<Payrun>, AppError> {
        sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns ORDER BY serial_id DESC")
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
    }

    async fn get_all_payslips(&self, pool: &PgPool) -> Result<Vec<Payslip>, AppError> {
        sqlx::query_as::<_, Payslip>("SELECT * FROM payroll.payslips ORDER BY serial_id DESC")
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
    }

    async fn get_payslips_for_payrun(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        sqlx::query_as::<_, Payslip>(
            "SELECT * FROM payroll.payslips WHERE payrun_uuid = $1 ORDER BY serial_id",
        )
        .bind(payrun_uuid)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    /*async fn create_pay_component(
        &self,
        pool: &PgPool,
        payload: &CreatePayComponent,
    ) -> Result<PayComponent, AppError> {
        sqlx::query_as::<_, PayComponent>(
            r#"
            INSERT INTO payroll.pay_components (
                code, name, component_kind, calc_method, default_rate,
                is_taxable, affects_social_security, statutory_role, gl_account_hint
            )
            VALUES (
                $1, $2, $3, $4, $5,
                COALESCE($6, true), COALESCE($7, false), $8, $9
            )
            RETURNING *
            "#,
        )
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.component_kind)
        .bind(&payload.calc_method)
        .bind(&payload.default_rate)
        .bind(payload.is_taxable)
        .bind(payload.affects_social_security)
        .bind(&payload.statutory_role)
        .bind(&payload.gl_account_hint)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }*/

async fn create_pay_component(
    &self,
    pool: &PgPool,
    payload: &CreatePayComponent,
) -> Result<PayComponent, AppError> {
    if payload.code.trim().is_empty() || payload.name.trim().is_empty() {
        return Err(AppError::Unprocessable(
            "Component code and name are required".into(),
        ));
    }

    if payload.default_rate.as_ref().is_some_and(|v| {
        v < &BigDecimal::from(0)
    }) {
        return Err(AppError::Unprocessable(
            "Component rate cannot be negative".into(),
        ));
    }

    if payload.rounding_scale < 0 || payload.rounding_scale > 6 {
        return Err(AppError::Unprocessable(
            "Rounding scale must be between 0 and 6".into(),
        ));
    }

    if payload.effective_from.zip(payload.effective_to)
        .is_some_and(|(from, to)| to < from)
    {
        return Err(AppError::Unprocessable(
            "Component effective end date cannot precede start date".into(),
        ));
    }

    sqlx::query_as::<_, PayComponent>(
        r#"
        INSERT INTO payroll.pay_components (
            code,
            name,
            component_kind,
            calc_method,
            default_rate,
            is_taxable,
            affects_social_security,
            statutory_role,
            gl_account_hint,
            sequence_no,
            rounding_scale,
            effective_from,
            effective_to
        )
        VALUES (
            $1,$2,$3,$4,$5,
            COALESCE($6, TRUE),
            COALESCE($7, TRUE),
            $8,$9,$10,$11,$12,$13
        )
        RETURNING *
        "#,
    )
    .bind(payload.code.trim().to_uppercase())
    .bind(payload.name.trim())
    .bind(&payload.component_kind)
    .bind(&payload.calc_method)
    .bind(&payload.default_rate)
    .bind(payload.is_taxable)
    .bind(payload.affects_social_security)
    .bind(&payload.statutory_role)
    .bind(&payload.gl_account_hint)
    .bind(payload.sequence_no)
    .bind(payload.rounding_scale)
    .bind(payload.effective_from)
    .bind(payload.effective_to)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

    async fn list_pay_components(&self, pool: &PgPool) -> Result<Vec<PayComponent>, AppError> {
        sqlx::query_as::<_, PayComponent>(
            "SELECT * FROM payroll.pay_components WHERE COALESCE(is_active, true) ORDER BY code",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

	async fn list_pay_components(
    	&self,
    	pool: &PgPool,
	) -> Result<Vec<PayComponent>, AppError> {
    	sqlx::query_as::<_, PayComponent>(
        	r#"
        	SELECT *
        	FROM payroll.pay_components
        	WHERE is_active = TRUE
        	ORDER BY sequence_no, code
        	"#,
    	)
    	.fetch_all(pool)
    	.await
    	.map_err(AppError::from)
	}			
}
/*
fn money(value: &str) -> BigDecimal {
    BigDecimal::from_str(value).expect("valid decimal constant")
}

fn zero() -> BigDecimal {
    BigDecimal::from(0)
}

fn round_money(value: &BigDecimal) -> BigDecimal {
    value.round(2)
}

fn positive(value: &BigDecimal) -> BigDecimal {
    if value < &zero() {
        zero()
    } else {
        value.clone()
    }
}

impl std::fmt::Display for PayrollCalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PayrollCalculationError {}

fn inclusive_days(start: NaiveDate, end: NaiveDate) -> i64 {
    (end - start).num_days() + 1
}

/// Prorates a salary using calendar days in the payroll period.
/// The overlap is inclusive at both ends.
pub fn prorate_salary(
    salary: &BigDecimal,
    period_start: NaiveDate,
    period_end: NaiveDate,
    contract_start: NaiveDate,
    contract_end: Option<NaiveDate>,
) -> Result<BigDecimal, PayrollCalculationError> {
    if period_end < period_start {
        return Err(PayrollCalculationError::InvalidPeriod);
    }

    if let Some(end) = contract_end {
        if end < contract_start {
            return Err(PayrollCalculationError::InvalidContractPeriod);
        }
    }

    if salary < &zero() {
        return Err(PayrollCalculationError::NegativeBasicSalary);
    }

    let effective_start = period_start.max(contract_start);
    let effective_end = contract_end
        .map(|end| period_end.min(end))
        .unwrap_or(period_end);

    if effective_end < effective_start {
        return Ok(zero());
    }

    let period_days = inclusive_days(period_start, period_end);
    let eligible_days = inclusive_days(effective_start, effective_end);

    Ok(round_money(
        &(salary * BigDecimal::from(eligible_days)
            / BigDecimal::from(period_days)),
    ))
}

/// Progressive tax calculation using the configured band table.
/// base_tax is the cumulative tax payable at lower_bound.
pub fn progressive_tax(
    taxable_income: &BigDecimal,
    bands: &[TaxBand],
) -> Result<BigDecimal, PayrollCalculationError> {
    if taxable_income <= &zero() {
        return Ok(zero());
    }

    if bands.is_empty() {
        return Err(PayrollCalculationError::MissingTaxBands);
    }

    let mut ordered = bands.to_vec();
    ordered.sort_by(|a, b| a.lower_bound.cmp(&b.lower_bound));

    let mut tax = zero();

    for band in &ordered {
        if taxable_income <= &band.lower_bound {
            continue;
        }

        let taxable_upper = match &band.upper_bound {
            Some(upper) if taxable_income > upper => upper.clone(),
            Some(upper) => taxable_income.clone().min(upper.clone()),
            None => taxable_income.clone(),
        };

        let taxable_in_band = positive(&(taxable_upper - &band.lower_bound));

        tax += &band.base_tax
            + (taxable_in_band * &band.marginal_rate);

        // The next band contains the cumulative base tax.
        // Do not add the previous band's tax a second time.
        break;
    }

    // Find the highest band whose lower bound is not greater than income.
    let applicable = ordered
        .iter()
        .filter(|b| taxable_income >= &b.lower_bound)
        .last()
        .ok_or(PayrollCalculationError::MissingTaxBands)?;

    let taxable_in_band = match &applicable.upper_bound {
        Some(upper) => taxable_income.clone().min(upper.clone())
            - &applicable.lower_bound,
        None => taxable_income - &applicable.lower_bound,
    };

    Ok(round_money(
        &(&applicable.base_tax
            + positive(&taxable_in_band) * &applicable.marginal_rate),
    ))
}

fn component_amount(
    component: &ComponentInput,
    basic: &BigDecimal,
    gross_before_component: &BigDecimal,
) -> Result<BigDecimal, PayrollCalculationError> {
    if component.amount < zero() {
        return Err(PayrollCalculationError::InvalidComponentAmount(
            component.code.clone(),
        ));
    }

    let calculated = match component.method {
        CalculationMethod::Fixed | CalculationMethod::Manual => {
            component.amount.clone()
        }
        CalculationMethod::PercentOfBasic => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(component.code.clone())
            })?;
            basic * rate
        }
        CalculationMethod::PercentOfGross => {
            let rate = component.rate.as_ref().ok_or_else(|| {
                PayrollCalculationError::InvalidComponentRate(component.code.clone())
            })?;
            gross_before_component * rate
        }
        CalculationMethod::External => {
            component.amount.clone()
        }
    };

    Ok(round_money(&calculated))
}

fn nssf_is_applicable(
    profile: &EmployeeTaxProfile,
    period_end: NaiveDate,
    minimum_age: Option<u8>,
    maximum_age: Option<u8>,
) -> bool {
    if !profile.nssf_covered || profile.nssf_exempt {
        return false;
    }

    let Some(dob) = profile.date_of_birth else {
        // Missing date of birth must be surfaced as a data-quality issue
        // by the caller; do not silently infer eligibility.
        return false;
    };

    let mut age = period_end.year() - dob.year();
    if (period_end.month(), period_end.day()) < (dob.month(), dob.day()) {
        age -= 1;
    }

    if let Some(minimum) = minimum_age {
        if age < i32::from(minimum) {
            return false;
        }
    }

    if let Some(maximum) = maximum_age {
        if age > i32::from(maximum) {
            return false;
        }
    }

    true
}

pub fn calculate_payroll(
    input: &PayrollInput,
) -> Result<PayrollResult, PayrollCalculationError> {
    if input.period_end < input.period_start {
        return Err(PayrollCalculationError::InvalidPeriod);
    }

    if input.tax_profile.tax_residency != "RESIDENT"
        && input.tax_profile.tax_residency != "NON_RESIDENT"
    {
        return Err(PayrollCalculationError::InvalidTaxResidency);
    }

    let basic = prorate_salary(
        &input.basic_salary,
        input.period_start,
        input.period_end,
        input.contract_start,
        input.contract_end,
    )?;

    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut gross = basic.clone();
    let mut taxable = basic.clone();
    let mut nssf_base = basic.clone();
    let mut other_deductions = zero();
    let mut employer_costs = zero();

    for component in &input.components {
        if !seen.insert(component.code.to_uppercase()) {
            return Err(PayrollCalculationError::DuplicateComponent(
                component.code.clone(),
            ));
        }

        let basis = match component.method {
            CalculationMethod::PercentOfBasic => basic.clone(),
            CalculationMethod::PercentOfGross => gross.clone(),
            _ => component.amount.clone(),
        };

        let amount = component_amount(component, &basic, &gross)?;

        match component.kind {
            ComponentKind::Earning => {
                gross += &amount;
                if component.taxable {
                    taxable += &amount;
                }
                if component.included_in_nssf_base {
                    nssf_base += &amount;
                }
            }
            ComponentKind::Deduction => {
                other_deductions += &amount;
            }
            ComponentKind::EmployerCost => {
                employer_costs += &amount;
            }
        }

        items.push(CalculatedItem {
            component_uuid: component.component_uuid,
            code: component.code.clone(),
            name: component.name.clone(),
            kind: component.kind,
            method: component.method,
            basis_amount: basis,
            rate: component.rate.clone(),
            unrounded_amount: amount.clone(),
            amount,
            taxable: component.taxable,
            included_in_nssf_base: component.included_in_nssf_base,
        });
    }

    let paye = progressive_tax(&taxable, &input.tax_bands)?;

    let nssf_applicable = nssf_is_applicable(
        &input.tax_profile,
        input.period_end,
        input.nssf_minimum_age,
        input.nssf_maximum_age,
    );

    let employee_nssf = if nssf_applicable {
        round_money(&(&nssf_base * &input.nssf_employee_rate))
    } else {
        zero()
    };

    let employer_nssf = if nssf_applicable {
        round_money(&(&nssf_base * &input.nssf_employer_rate))
    } else {
        zero()
    };

    let net = round_money(
        &(&gross - &paye - &employee_nssf - &other_deductions),
    );

    if net < zero() {
        return Err(PayrollCalculationError::NegativeNetPay);
    }

    let employer_cost_total = round_money(
        &(&gross + &employer_nssf + &employer_costs),
    );

    Ok(PayrollResult {
        employee_uuid: input.employee_uuid,
        basic_pay: basic,
        gross_pay: round_money(&gross),
        taxable_earnings: round_money(&taxable),
        paye,
        employee_nssf,
        employer_nssf,
        other_deductions: round_money(&other_deductions),
        employer_cost_total,
        net_pay: net,
        items,
    })
}
*/
