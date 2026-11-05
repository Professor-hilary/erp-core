// src/features/payroll/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::payroll::{
        ComponentAssignmentRow, CreateComponentAssignment, CreatePayComponent, CreatePayrun,
        CreatePayslip, EligibleEmployeeRow, EmployeeComponentAssignment,
        EmployeePaymentBatch, EmployeePayrollContext, EmployeeTaxPeriodRow,
        LeaveEncashmentRow, NssfRuleRow, PayComponent, Payrun, Payslip, PayslipCalculationItem,
        PostPayrun, RemitStatutory, SettleEmployees, StatutoryBandRow, StatutoryRemittance,
    },
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use serde_json::json;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

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

    // Component assignments
    async fn create_component_assignment(
        &self,
        pool: &PgPool,
        payload: &CreateComponentAssignment,
    ) -> Result<EmployeeComponentAssignment, AppError>;
    async fn list_component_assignments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmployeeComponentAssignment>, AppError>;

    // Payslip breakdown
    async fn get_payslip_calculation_items(
        &self,
        pool: &PgPool,
        payslip_uuid: Uuid,
    ) -> Result<Vec<PayslipCalculationItem>, AppError>;

    // Settlement
    async fn settle_employees(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &SettleEmployees,
    ) -> Result<EmployeePaymentBatch, AppError>;
    async fn get_payment_batch_for_payrun(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Option<EmployeePaymentBatch>, AppError>;

    // Statutory remittance
    async fn create_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
        due_date: NaiveDate,
    ) -> Result<Vec<StatutoryRemittance>, AppError>;
    async fn remit_statutory(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &RemitStatutory,
    ) -> Result<StatutoryRemittance, AppError>;
    async fn list_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Option<Uuid>,
    ) -> Result<Vec<StatutoryRemittance>, AppError>;
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
                    payrun_uuid, employee_uuid, gross_pay, basic_pay,
                    tax_deducted, social_security, employer_social_security,
                    other_deductions, payment_method, bank_account
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
                        return Err(AppError::Unprocessable(
                            "Invalid payslip item type".into(),
                        ));
                    }
                    sqlx::query(
                        r#"
                        INSERT INTO payroll.payslip_items (
                            payslip_uuid, item_type, description, amount, component_uuid
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
        let rows = sqlx::query_as::<_, EligibleEmployeeRow>(
            r#"
            SELECT
                e.uuid AS employee_uuid,
                c.basic_salary,
                pp.bank_account_number,
                pp.payment_method
            FROM hr.employees e
            INNER JOIN hr.employment_contracts c
                ON c.employee_uuid = e.uuid AND c.status = 'Active'
            LEFT JOIN hr.employee_payment_profiles pp
                ON pp.employee_uuid = e.uuid
            WHERE hr.is_payroll_eligible(e.uuid, $1, $2)
              AND c.basic_salary IS NOT NULL
              AND c.basic_salary > 0
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
        use crate::features::payroll::calculation::{
            calculate_payroll, CalculationMethod, ComponentInput, ComponentKind, EmployeeTaxProfile,
            PayrollInput, TaxBand,
        };
        use chrono::Datelike;

        let mut tx = pool.begin().await?;

        // 1. Lock the payrun
        let payrun: Payrun = sqlx::query_as(
            r#"
            SELECT * FROM payroll.payruns
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

        // Tax-month boundary: a single payrun must not span two calendar months.
        if payrun.pay_period_start.year() != payrun.pay_period_end.year()
            || payrun.pay_period_start.month() != payrun.pay_period_end.month()
        {
            return Err(AppError::Unprocessable(
                "Payrun period must fall within a single tax month; split the run".into(),
            ));
        }

        let slips: Vec<Payslip> = sqlx::query_as(
            r#"
            SELECT * FROM payroll.payslips
            WHERE payrun_uuid = $1
            ORDER BY serial_id
            FOR UPDATE
            "#,
        )
        .bind(payrun.uuid)
        .fetch_all(&mut *tx)
        .await?;

        if slips.is_empty() {
            return Err(AppError::Unprocessable(
                "Cannot process an empty payrun".into(),
            ));
        }

        let calculation_version = "payroll-engine-v1".to_string();
        let period_start = payrun.pay_period_start;
        let period_end = payrun.pay_period_end;
        let tax_year = period_end.year() as i16;
        let tax_month = period_end.month() as i16;

        for slip in &slips {
            // Skip already-calculated slips (idempotent re-process guard)
            if slip.calculated_at.is_some() && slip.calculation_version.is_some() {
                continue;
            }

            // 2. Load employee payroll context
            let ctx: EmployeePayrollContext = sqlx::query_as(
                r#"
                SELECT
                    e.uuid AS employee_uuid,
                    COALESCE(sp.date_of_birth, e.date_of_birth) AS date_of_birth,
                    COALESCE(sp.tax_residency, 'RESIDENT') AS tax_residency,
                    COALESCE(sp.nssf_covered, TRUE) AS nssf_covered,
                    COALESCE(sp.nssf_exempt, FALSE) AS nssf_exempt,
                    c.uuid AS contract_uuid,
                    c.start_date AS contract_start,
                    c.end_date AS contract_end,
                    c.basic_salary,
                    COALESCE(c.pay_frequency, e.pay_frequency, 'Monthly') AS pay_frequency,
                    pp.payment_method,
                    pp.bank_account_number
                FROM hr.employees e
                LEFT JOIN hr.employee_statutory_profiles sp ON sp.employee_uuid = e.uuid
                INNER JOIN hr.employment_contracts c
                    ON c.employee_uuid = e.uuid AND c.status = 'Active'
                LEFT JOIN hr.employee_payment_profiles pp ON pp.employee_uuid = e.uuid
                WHERE e.uuid = $1
                "#,
            )
            .bind(slip.employee_uuid)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                AppError::Unprocessable(format!(
                    "Employee {} not found for payslip",
                    slip.employee_uuid
                ))
            })?;

			if ctx.employee_uuid != slip.employee_uuid {
			    return Err(AppError::Unprocessable(
			        "Payroll context employee mismatch".into(),
			    ));
			}

            let mut basic_salary = ctx.basic_salary.clone().ok_or_else(|| {
                AppError::Unprocessable(format!(
                    "Employee {} has no active contract salary",
                    slip.employee_uuid
                ))
            })?;
            // Effective-dated amendments: use the latest salary amendment
            // whose effective_date is on or before the pay period end.
            if let Some(contract_uuid) = ctx.contract_uuid {
                let amended: Option<BigDecimal> = sqlx::query_scalar(
                    r#"
                    SELECT basic_salary
                    FROM hr.contract_amendments
                    WHERE contract_uuid = $1
                      AND basic_salary IS NOT NULL
                      AND effective_date <= $2
                    ORDER BY effective_date DESC, created_at DESC
                    LIMIT 1
                    "#,
                )
                .bind(contract_uuid)
                .bind(period_end)
                .fetch_optional(&mut *tx)
                .await?;
                if let Some(sal) = amended {
                    basic_salary = sal;
                }
            }
            if basic_salary <= BigDecimal::from(0) {
                return Err(AppError::Unprocessable(format!(
                    "Employee {} has non-positive basic salary",
                    slip.employee_uuid
                )));
            }
            let contract_start = ctx.contract_start.ok_or_else(|| {
                AppError::Unprocessable(format!(
                    "Employee {} contract has no start date",
                    slip.employee_uuid
                ))
            })?;
            // Cap service end by contract end and last_working_date from lifecycle
            let last_working: Option<chrono::NaiveDate> = sqlx::query_scalar(
                "SELECT last_working_date FROM hr.employee_lifecycle WHERE employee_uuid = $1",
            )
            .bind(slip.employee_uuid)
            .fetch_optional(&mut *tx)
            .await?
            .flatten();
            let contract_end = match (ctx.contract_end, last_working) {
                (Some(c), Some(l)) => Some(std::cmp::min(c, l)),
                (Some(c), None) => Some(c),
                (None, Some(l)) => Some(l),
                (None, None) => None,
            };

            // Pay-frequency aware period base:
            // basic_salary is treated as the amount for one full frequency period.
            // Convert to the payrun period amount before day-proration.
            let period_days = (period_end - period_start).num_days() + 1;
            let freq = ctx.pay_frequency.as_str();
            let frequency_days: i64 = match freq {
                "Weekly" => 7,
                "Bi-weekly" | "Biweekly" | "Bi-Weekly" => 14,
                _ => {
                    // Monthly (default): use actual calendar days of the end month
                    let y = period_end.year();
                    let m = period_end.month();
                    let last = chrono::NaiveDate::from_ymd_opt(
                        if m == 12 { y + 1 } else { y },
                        if m == 12 { 1 } else { m + 1 },
                        1,
                    )
                    .unwrap()
                    .pred_opt()
                    .unwrap();
                    last.day() as i64
                }
            };
            // Amount attributable to this payrun's calendar span if the employee
            // worked the whole period under a full-frequency salary.
            let period_salary_base = if frequency_days > 0 {
                &basic_salary * BigDecimal::from(period_days) / BigDecimal::from(frequency_days)
            } else {
                basic_salary.clone()
            };

            // 3. Load component assignments effective in the period
            let assignments: Vec<ComponentAssignmentRow> = sqlx::query_as(
                r#"
                SELECT
                    a.component_uuid,
                    pc.code,
                    pc.name,
                    pc.component_kind,
                    pc.calc_method,
                    a.amount,
                    COALESCE(a.rate, pc.default_rate) AS rate,
                    pc.is_taxable,
                    pc.affects_social_security,
                    pc.sequence_no,
                    pc.rounding_scale
                FROM payroll.employee_component_assignments a
                JOIN payroll.pay_components pc ON pc.uuid = a.component_uuid
                WHERE a.employee_uuid = $1
                  AND a.is_active
                  AND pc.is_active
                  AND a.effective_from <= $3
                  AND (a.effective_to IS NULL OR a.effective_to >= $2)
                  AND (pc.effective_from IS NULL OR pc.effective_from <= $3)
                  AND (pc.effective_to IS NULL OR pc.effective_to >= $2)
                ORDER BY pc.sequence_no, pc.code
                "#,
            )
            .bind(slip.employee_uuid)
            .bind(period_start)
            .bind(period_end)
            .fetch_all(&mut *tx)
            .await?;

            // 4a. Manual payslip items (benefits/deductions entered at prepare time)
            let manual_items: Vec<(String, String, BigDecimal, Option<Uuid>)> = sqlx::query_as(
                r#"
                SELECT item_type, COALESCE(description, item_type), amount, component_uuid
                FROM payroll.payslip_items
                WHERE payslip_uuid = $1
                "#,
            )
            .bind(slip.uuid)
            .fetch_all(&mut *tx)
            .await?;

            // 4. Leave encashments pending for this employee
            let encashments: Vec<LeaveEncashmentRow> = sqlx::query_as(
                r#"
                SELECT
                    le.uuid,
                    le.amount,
                    le.days,
                    lt.code AS leave_type_code
                FROM hr.leave_encashments le
                LEFT JOIN hr.leave_types lt ON lt.uuid = le.leave_type_uuid
                WHERE le.employee_uuid = $1
                  AND le.status = 'Pending'
                  AND (le.payrun_uuid IS NULL OR le.payrun_uuid = $2)
                "#,
            )
            .bind(slip.employee_uuid)
            .bind(payrun.uuid)
            .fetch_all(&mut *tx)
            .await?;

            // 5. Build ComponentInput list
            let mut components: Vec<ComponentInput> = Vec::new();

            for a in &assignments {
                let kind = match a.component_kind.as_str() {
                    "EARNING" => ComponentKind::Earning,
                    "DEDUCTION" => ComponentKind::Deduction,
                    "EMPLOYER_COST" => ComponentKind::EmployerCost,
                    _ => {
                        return Err(AppError::Unprocessable(format!(
                            "Unknown component_kind {}",
                            a.component_kind
                        )))
                    }
                };
                let method = match a.calc_method.as_str() {
                    "FIXED" => CalculationMethod::Fixed,
                    "PERCENT_OF_BASIC" => CalculationMethod::PercentOfBasic,
                    "PERCENT_OF_GROSS" => CalculationMethod::PercentOfGross,
                    "PERCENT_OF_TAXABLE_EARNINGS" => CalculationMethod::PercentOfTaxableEarnings,
                    "MANUAL" => CalculationMethod::Manual,
                    "EXTERNAL" => CalculationMethod::External,
                    _ => {
                        return Err(AppError::Unprocessable(format!(
                            "Unknown calc_method {}",
                            a.calc_method
                        )))
                    }
                };

                components.push(ComponentInput {
                    component_uuid: Some(a.component_uuid),
                    code: a.code.clone(),
                    name: a.name.clone(),
                    kind,
                    method,
                    amount: a.amount.clone().unwrap_or_else(|| BigDecimal::from(0)),
                    rate: a.rate.clone(),
                    taxable: a.is_taxable,
                    included_in_nssf_base: a.affects_social_security,
                    sequence_no: a.sequence_no,
                    rounding_scale: a.rounding_scale as i64,
                });
            }

            // Manual payslip benefits / deductions
            for (idx, (item_type, description, amount, component_uuid)) in
                manual_items.iter().enumerate()
            {
                let kind = if item_type.eq_ignore_ascii_case("Deduction") {
                    ComponentKind::Deduction
                } else {
                    ComponentKind::Earning
                };
                components.push(ComponentInput {
                    component_uuid: *component_uuid,
                    code: format!("MANUAL_{}_{}", item_type.to_uppercase(), idx + 1),
                    name: description.clone(),
                    kind,
                    method: CalculationMethod::Fixed,
                    amount: amount.clone(),
                    rate: None,
                    taxable: kind == ComponentKind::Earning,
                    included_in_nssf_base: kind == ComponentKind::Earning,
                    sequence_no: 40,
                    rounding_scale: 2,
                });
            }

            // Leave encashment as taxable earning
            for (idx, enc) in encashments.iter().enumerate() {
				if enc.days <= BigDecimal::from(0) {
			        return Err(AppError::Unprocessable(format!(
			            "Leave encashment {} has non-positive days",
			            enc.uuid
			        )));
			    }
			    let daily_rate = &enc.amount / &enc.days;
			    let type_label = enc
			        .leave_type_code
			        .as_ref()
			        .map(|c| format!(" ({c})"))
			        .unwrap_or_default();

                components.push(ComponentInput {
                    component_uuid: None,
                    code: format!("LEAVE_ENCASH_{}", idx + 1),
                    name: format!(
                        "Leave encashment{} — {} day(s) @ {}",
						type_label, enc.days, daily_rate
                    ),
                    kind: ComponentKind::Earning,
                    method: CalculationMethod::Fixed,
                    amount: enc.amount.clone(),
                    rate: Some(daily_rate),
                    taxable: true,
                    included_in_nssf_base: true,
                    sequence_no: 50,
                    rounding_scale: 2,
                });
            }

            // 6. Load statutory PAYE bands
            let residency = if ctx.tax_residency.eq_ignore_ascii_case("NON_RESIDENT") {
                "NON_RESIDENT"
            } else {
                "RESIDENT"
            };

            let bands: Vec<StatutoryBandRow> = sqlx::query_as(
                r#"
                SELECT b.lower_bound, b.upper_bound, b.base_tax, b.marginal_rate, b.sequence_no
                FROM payroll.statutory_rule_sets r
                JOIN payroll.statutory_rule_bands b ON b.rule_set_uuid = r.uuid
                WHERE r.jurisdiction_code = 'UG'
                  AND r.rule_code = 'PAYE'
                  AND r.employee_class = $1
                  AND r.is_active
                  AND r.effective_from <= $2
                  AND (r.effective_to IS NULL OR r.effective_to >= $2)
                ORDER BY b.sequence_no
                "#,
            )
            .bind(residency)
            .bind(period_end)
            .fetch_all(&mut *tx)
            .await?;

            if bands.is_empty() {
                return Err(AppError::Unprocessable(format!(
                    "No PAYE bands found for {} on {}",
                    residency, period_end
                )));
            }

			let mut last_seq: Option<i16> = None;
			let mut tax_bands: Vec<TaxBand> = Vec::with_capacity(bands.len());
			for b in bands {
			    if let Some(prev) = last_seq {
			        if b.sequence_no <= prev {
			            return Err(AppError::Unprocessable(format!(
			                "PAYE bands out of sequence: {} after {}",
			                b.sequence_no, prev
			            )));
			        }
			    }
			    last_seq = Some(b.sequence_no);
			    tax_bands.push(TaxBand {
			        lower_bound: b.lower_bound,
			        upper_bound: b.upper_bound,
			        base_tax: b.base_tax,
			        marginal_rate: b.marginal_rate,
			    });
			}

            /*let tax_bands: Vec<TaxBand> = bands
                .into_iter()
                .map(|b| TaxBand {
                    lower_bound: b.lower_bound,
                    upper_bound: b.upper_bound,
                    base_tax: b.base_tax,
                    marginal_rate: b.marginal_rate,
                })
                .collect();*/

            // 7. Load NSSF rates
            let nssf: Option<NssfRuleRow> = sqlx::query_as(
                r#"
                SELECT
                    r.uuid AS rule_set_uuid,
                    c.employee_rate,
                    c.employer_rate,
                    c.minimum_age,
                    c.maximum_age
                FROM payroll.statutory_rule_sets r
                JOIN payroll.statutory_contribution_rules c ON c.rule_set_uuid = r.uuid
                WHERE r.jurisdiction_code = 'UG'
                  AND r.rule_code = 'NSSF'
                  AND r.is_active
                  AND r.effective_from <= $1
                  AND (r.effective_to IS NULL OR r.effective_to >= $1)
                LIMIT 1
                "#,
            )
            .bind(period_end)
            .fetch_optional(&mut *tx)
            .await?;

            let (nssf_employee_rate, nssf_employer_rate, nssf_min_age, nssf_max_age, nssf_rule_set_uuid) =
                match nssf {
                    Some(n) => (
                        n.employee_rate,
                        n.employer_rate,
                        n.minimum_age.map(|a| a as u8),
                        n.maximum_age.map(|a| a as u8),
						Some(n.rule_set_uuid),
                    ),
                    None => (
                        BigDecimal::from(5) / BigDecimal::from(100),
                        BigDecimal::from(10) / BigDecimal::from(100),
                        Some(16),
                        Some(55),
						None
                    ),
                };

            // 8. Build calculation input and run the engine
            let input = PayrollInput {
                employee_uuid: slip.employee_uuid,
                basic_salary: period_salary_base,
                period_start,
                period_end,
                contract_start,
                contract_end,
                components,
                tax_profile: EmployeeTaxProfile {
                    tax_residency: residency.to_string(),
                    nssf_covered: ctx.nssf_covered,
                    nssf_exempt: ctx.nssf_exempt,
                    date_of_birth: ctx.date_of_birth,
                },
                tax_bands,
                nssf_employee_rate: nssf_employee_rate.clone(),
                nssf_employer_rate: nssf_employer_rate.clone(),
                nssf_minimum_age: nssf_min_age,
                nssf_maximum_age: nssf_max_age,
            };

            let mut result = calculate_payroll(&input).map_err(|e| {
                AppError::Unprocessable(format!(
                    "Payroll calculation failed for employee {}: {}",
                    slip.employee_uuid, e
                ))
            })?;

            // 9. Cumulative PAYE within the tax month
            // Guard: refuse out-of-order processing for the same employee/tax month.
            let later_conflict: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(*)
                FROM payroll.payslips ps
                JOIN payroll.payruns pr ON pr.uuid = ps.payrun_uuid
                WHERE ps.employee_uuid = $1
                  AND ps.calculated_at IS NOT NULL
                  AND pr.status IN ('Processed', 'Posted')
                  AND EXTRACT(YEAR FROM pr.pay_period_end) = $2
                  AND EXTRACT(MONTH FROM pr.pay_period_end) = $3
                  AND pr.pay_period_end > $4
                  AND pr.uuid <> $5
                "#,
            )
            .bind(slip.employee_uuid)
            .bind(tax_year)
            .bind(tax_month)
            .bind(period_end)
            .bind(payrun.uuid)
            .fetch_one(&mut *tx)
            .await?;

            if later_conflict > 0 {
                return Err(AppError::Unprocessable(format!(
                    "Employee {}: a later payrun in tax month {}-{} is already processed. Process payruns in chronological order.",
                    slip.employee_uuid, tax_year, tax_month
                )));
            }

            // Compute tax on (prior taxable + this taxable), subtract prior withheld.
            let prior: Option<EmployeeTaxPeriodRow> = sqlx::query_as(
                r#"
                SELECT uuid, taxable_income, paye_due, paye_withheld
                FROM payroll.employee_tax_periods
                WHERE employee_uuid = $1
                  AND tax_year = $2
                  AND tax_month = $3
                FOR UPDATE
                "#,
            )
            .bind(slip.employee_uuid)
            .bind(tax_year)
            .bind(tax_month)
            .fetch_optional(&mut *tx)
            .await?;

            let (prior_taxable, prior_withheld, prior_paye_due, tax_period_uuid) = match &prior {
                Some(p) => (p.taxable_income.clone(), p.paye_due.clone(), p.paye_withheld.clone(), Some(p.uuid)),
                None => (BigDecimal::from(0), BigDecimal::from(0), BigDecimal::from(0), None),
            };

			if prior_withheld > prior_paye_due + BigDecimal::from(1) {
			    return Err(AppError::Unprocessable(format!(
			        "Employee {}: tax period inconsistent (withheld {} > due {}). Repair tax period before processing.",
			        slip.employee_uuid, prior_withheld, prior_paye_due
			    )));
			}

            let cumulative_taxable = &prior_taxable + &result.taxable_earnings;
            let cumulative_paye = crate::features::payroll::calculation::progressive_tax(
                &cumulative_taxable,
                &input.tax_bands,
            )
            .map_err(|e| AppError::Unprocessable(format!("Cumulative PAYE failed: {e}")))?;

            let period_paye = &cumulative_paye - &prior_withheld;
            if period_paye < BigDecimal::from(0) {
                // Over-withholding from prior runs in the month — force review
                return Err(AppError::Unprocessable(format!(
                    "Employee {}: cumulative PAYE already exceeds tax due for the month (prior withheld {}, cumulative due {}). Reprocess in chronological order or reverse the later run.",
                    slip.employee_uuid, prior_withheld, cumulative_paye
                )));
            }
            result.paye = period_paye.clone();
            // Recompute net after PAYE adjustment — do NOT clamp; surface the exception
            result.net_pay = &result.gross_pay
                - &result.paye
                - &result.employee_nssf
                - &result.other_deductions;
            if result.net_pay < BigDecimal::from(0) {
                return Err(AppError::Unprocessable(format!(
                    "Employee {}: negative net pay after cumulative PAYE (gross {}, PAYE {}, NSSF {}, other {}). Correct inputs before processing.",
                    slip.employee_uuid,
                    result.gross_pay,
                    result.paye,
                    result.employee_nssf,
                    result.other_deductions
                )));
            }

            // 10. Persist calculation results onto the payslip
            let snapshot = json!({
                "version": calculation_version,
                "basic_pay": result.basic_pay,
                "gross_pay": result.gross_pay,
                "nssf_contribution_base": result.nssf_contribution_base,
                "taxable_earnings": result.taxable_earnings,
                "paye": result.paye,
                "employee_nssf": result.employee_nssf,
                "employer_nssf": result.employer_nssf,
                "other_deductions": result.other_deductions,
                "employer_cost_total": result.employer_cost_total,
                "net_pay": result.net_pay,
                "cumulative_taxable": cumulative_taxable,
                "cumulative_paye": cumulative_paye,
                "prior_withheld": prior_withheld,
                "items": result.items,
            });

            sqlx::query(
                r#"
                UPDATE payroll.payslips SET
                    basic_pay = $2,
                    gross_pay = $3,
                    allowances_total = $4,
                    taxable_earnings = $5,
                    tax_deducted = $6,
                    social_security = $7,
                    employer_social_security = $8,
                    other_deductions = $9,
                    employer_cost_total = $10,
                    calculation_version = $11,
                    calculated_at = NOW(),
                    calculation_snapshot = $12,
					payment_method = COALESCE($13, payment_method),
				    bank_account = COALESCE($14, bank_account),
                    updated_at = NOW()
                WHERE uuid = $1
                "#,
            )
            .bind(slip.uuid)
            .bind(&result.basic_pay)
            .bind(&result.gross_pay)
            .bind(&result.gross_pay - &result.basic_pay) // simple allowances proxy
            .bind(&result.taxable_earnings)
            .bind(&result.paye)
            .bind(&result.employee_nssf)
            .bind(&result.employer_nssf)
            .bind(&result.other_deductions)
            .bind(&result.employer_cost_total)
            .bind(&calculation_version)
            .bind(&snapshot)
			.bind(&ctx.payment_method)
			.bind(&ctx.bank_account_number)
            .execute(&mut *tx)
            .await?;

            // 11. Immutable calculation lines
            // Clear any previous (should be empty)
            sqlx::query("DELETE FROM payroll.payslip_calculation_items WHERE payslip_uuid = $1")
                .bind(slip.uuid)
                .execute(&mut *tx)
                .await?;

            for item in &result.items {
                sqlx::query(
                    r#"
                    INSERT INTO payroll.payslip_calculation_items (
                        payslip_uuid, component_uuid, item_code, item_name, item_kind,
                        calculation_method, basis_amount, rate, quantity,
                        unrounded_amount, amount, taxable, included_in_nssf_base
                    )
                    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,1,$9,$10,$11,$12)
                    "#,
                )
                .bind(slip.uuid)
                .bind(item.component_uuid)
                .bind(&item.code)
                .bind(&item.name)
                .bind(match item.kind {
                    ComponentKind::Earning => "EARNING",
                    ComponentKind::Deduction => "DEDUCTION",
                    ComponentKind::EmployerCost => "EMPLOYER_COST",
                })
                .bind(format!("{:?}", item.method).to_uppercase())
                .bind(&item.basis_amount)
                .bind(&item.rate)
                .bind(&item.unrounded_amount)
                .bind(&item.amount)
                .bind(item.taxable)
                .bind(item.included_in_nssf_base)
                .execute(&mut *tx)
                .await?;
            }

            // Also record statutory lines for audit
            sqlx::query(
                r#"
                INSERT INTO payroll.payslip_calculation_items (
                    payslip_uuid, item_code, item_name, item_kind,
                    calculation_method, basis_amount, rate, quantity,
                    unrounded_amount, amount, taxable, included_in_nssf_base
                )
                VALUES
                    ($1, 'PAYE', 'PAYE', 'DEDUCTION', 'PROGRESSIVE_BANDS', $2, NULL, 1, $3, $3, FALSE, FALSE),
                    ($1, 'NSSF_EE', 'Employee NSSF', 'DEDUCTION', 'PERCENT_OF_BASE', $4, $5, 1, $6, $6, FALSE, FALSE),
                    ($1, 'NSSF_ER', 'Employer NSSF', 'EMPLOYER_COST', 'PERCENT_OF_BASE', $4, $7, 1, $8, $8, FALSE, FALSE)
                "#,
            )
            .bind(slip.uuid)
            .bind(&result.taxable_earnings)
            .bind(&result.paye)
            .bind(&result.nssf_contribution_base) // actual NSSF base
            .bind(&nssf_employee_rate.clone())
            .bind(&result.employee_nssf)
            .bind(&nssf_employer_rate.clone())
            .bind(&result.employer_nssf)
            .execute(&mut *tx)
            .await?;

            // 12. Update / create tax period + entry
            let tax_period_id = if let Some(id) = tax_period_uuid {
                sqlx::query(
                    r#"
                    UPDATE payroll.employee_tax_periods SET
                        taxable_income = $2,
                        paye_due = $3,
                        paye_withheld = $4,
                        updated_at = NOW()
                    WHERE uuid = $1
                    "#,
                )
                .bind(id)
                .bind(&cumulative_taxable)
                .bind(&cumulative_paye)
                .bind(&prior_withheld + &result.paye)
                .execute(&mut *tx)
                .await?;
                id
            } else {
                sqlx::query_scalar(
                    r#"
                    INSERT INTO payroll.employee_tax_periods (
                        employee_uuid, tax_year, tax_month,
                        taxable_income, paye_due, paye_withheld
                    )
                    VALUES ($1,$2,$3,$4,$5,$6)
                    RETURNING uuid
                    "#,
                )
                .bind(slip.employee_uuid)
                .bind(tax_year)
                .bind(tax_month)
                .bind(&result.taxable_earnings)
                .bind(&result.paye)
                .bind(&result.paye)
                .fetch_one(&mut *tx)
                .await?
            };

            sqlx::query(
                r#"
                INSERT INTO payroll.employee_tax_period_entries (
                    employee_tax_period_uuid, payslip_uuid, taxable_income, paye_withheld
                )
                VALUES ($1,$2,$3,$4)
                ON CONFLICT (payslip_uuid) DO UPDATE SET
                    taxable_income = EXCLUDED.taxable_income,
                    paye_withheld = EXCLUDED.paye_withheld
                "#,
            )
            .bind(tax_period_id)
            .bind(slip.uuid)
            .bind(&result.taxable_earnings)
            .bind(&result.paye)
            .execute(&mut *tx)
            .await?;

            // 13. Mark leave encashments as included
            for enc in &encashments {
                sqlx::query(
                    r#"
                    UPDATE hr.leave_encashments
                    SET status = 'Included_In_Payrun', payrun_uuid = $2
                    WHERE uuid = $1 AND status = 'Pending'
                    "#,
                )
                .bind(enc.uuid)
                .bind(payrun.uuid)
                .execute(&mut *tx)
                .await?;
            }
        }

        // 14. Final validation: every slip must now be calculated
        let uncalculated: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM payroll.payslips
            WHERE payrun_uuid = $1
              AND (calculated_at IS NULL OR calculation_version IS NULL)
            "#,
        )
        .bind(payrun.uuid)
        .fetch_one(&mut *tx)
        .await?;

        if uncalculated > 0 {
            return Err(AppError::Unprocessable(format!(
                "{uncalculated} payslip(s) still uncalculated after processing"
            )));
        }

        let invalid: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM payroll.payslips
            WHERE payrun_uuid = $1
              AND (gross_pay < 0 OR COALESCE(tax_deducted,0) < 0
                   OR COALESCE(social_security,0) < 0
                   OR COALESCE(other_deductions,0) < 0
                   OR net_pay < 0)
            "#,
        )
        .bind(payrun.uuid)
        .fetch_one(&mut *tx)
        .await?;

        if invalid > 0 {
            return Err(AppError::Unprocessable(
                "Payrun contains invalid payslip amounts after calculation".into(),
            ));
        }

        // 15. Lock the payrun
        let updated = sqlx::query_as::<_, Payrun>(
            r#"
            UPDATE payroll.payruns
            SET status = 'Processed',
                processed_at = NOW(),
                updated_at = NOW()
            WHERE uuid = $1 AND status = 'Pending'
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
        let employer_nssf_expense = payload
            .employer_social_security_expense_id
            .ok_or_else(|| {
                AppError::Unprocessable("Employer NSSF expense account is required".into())
            })?;
        let payable = payload.payroll_payable.ok_or_else(|| {
            AppError::Unprocessable("Payroll payable account is required".into())
        })?;

        let mut tx = pool.begin().await?;

        sqlx::query(
            r#"
            SELECT payroll.post_payrun($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
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
        .bind(payload.other_deductions_payable_id)
        .bind(payload.other_employer_cost_expense_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

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
        if payload
            .default_rate
            .as_ref()
            .is_some_and(|v| v < &BigDecimal::from(0))
        {
            return Err(AppError::Unprocessable(
                "Component rate cannot be negative".into(),
            ));
        }
        if payload.rounding_scale < 0 || payload.rounding_scale > 6 {
            return Err(AppError::Unprocessable(
                "Rounding scale must be between 0 and 6".into(),
            ));
        }
        if payload
            .effective_from
            .zip(payload.effective_to)
            .is_some_and(|(from, to)| to < from)
        {
            return Err(AppError::Unprocessable(
                "Component effective end date cannot precede start date".into(),
            ));
        }

        sqlx::query_as::<_, PayComponent>(
            r#"
            INSERT INTO payroll.pay_components (
                code, name, component_kind, calc_method, default_rate,
                is_taxable, affects_social_security, statutory_role, gl_account_hint,
                sequence_no, rounding_scale, effective_from, effective_to
            )
            VALUES (
                $1,$2,$3,$4,$5,
                COALESCE($6, TRUE), COALESCE($7, TRUE), $8,$9,
                $10,$11,$12,$13
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
            r#"
            SELECT * FROM payroll.pay_components
            WHERE is_active = TRUE
            ORDER BY sequence_no, code
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
    async fn create_component_assignment(
        &self,
        pool: &PgPool,
        payload: &CreateComponentAssignment,
    ) -> Result<EmployeeComponentAssignment, AppError> {
        if payload.amount.is_none() && payload.rate.is_none() {
            return Err(AppError::Unprocessable(
                "Assignment requires either amount or rate".into(),
            ));
        }
        if payload
            .effective_to
            .is_some_and(|to| to < payload.effective_from)
        {
            return Err(AppError::Unprocessable(
                "effective_to cannot precede effective_from".into(),
            ));
        }

        sqlx::query_as::<_, EmployeeComponentAssignment>(
            r#"
            INSERT INTO payroll.employee_component_assignments (
                employee_uuid, component_uuid, amount, rate,
                calculation_basis, effective_from, effective_to
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.component_uuid)
        .bind(&payload.amount)
        .bind(&payload.rate)
        .bind(&payload.calculation_basis)
        .bind(payload.effective_from)
        .bind(payload.effective_to)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_component_assignments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmployeeComponentAssignment>, AppError> {
        if let Some(emp) = employee_uuid {
            sqlx::query_as::<_, EmployeeComponentAssignment>(
                r#"
                SELECT * FROM payroll.employee_component_assignments
                WHERE employee_uuid = $1 AND is_active = TRUE
                ORDER BY effective_from DESC
                "#,
            )
            .bind(emp)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        } else {
            sqlx::query_as::<_, EmployeeComponentAssignment>(
                r#"
                SELECT * FROM payroll.employee_component_assignments
                WHERE is_active = TRUE
                ORDER BY employee_uuid, effective_from DESC
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        }
    }

    async fn get_payslip_calculation_items(
        &self,
        pool: &PgPool,
        payslip_uuid: Uuid,
    ) -> Result<Vec<PayslipCalculationItem>, AppError> {
        sqlx::query_as::<_, PayslipCalculationItem>(
            r#"
            SELECT * FROM payroll.payslip_calculation_items
            WHERE payslip_uuid = $1
            ORDER BY item_kind, item_code
            "#,
        )
        .bind(payslip_uuid)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    async fn settle_employees(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &SettleEmployees,
    ) -> Result<EmployeePaymentBatch, AppError> {
        let mut tx = pool.begin().await?;

        let payrun: Payrun = sqlx::query_as(
            r#"
            SELECT * FROM payroll.payruns
            WHERE uuid = $1
            FOR UPDATE
            "#,
        )
        .bind(payload.payrun_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Payrun not found".into()))?;

        if payrun.status != "Posted" {
            return Err(AppError::Unprocessable(
                "Payrun must be Posted before settling employees".into(),
            ));
        }

        // Idempotent: return existing batch if already created
        if let Some(existing) = sqlx::query_as::<_, EmployeePaymentBatch>(
            "SELECT * FROM payroll.employee_payment_batches WHERE payrun_uuid = $1",
        )
        .bind(payload.payrun_uuid)
        .fetch_optional(&mut *tx)
        .await?
        {
            if existing.status == "PAID" {
                return Ok(existing);
            }
        }

        let slips: Vec<Payslip> = sqlx::query_as(
            r#"
            SELECT * FROM payroll.payslips
            WHERE payrun_uuid = $1 AND status = 'Pending'
            FOR UPDATE
            "#,
        )
        .bind(payload.payrun_uuid)
        .fetch_all(&mut *tx)
        .await?;

        if slips.is_empty() {
            return Err(AppError::Unprocessable(
                "No unsettled payslips for this payrun".into(),
            ));
        }

        let total_net: BigDecimal = slips.iter().fold(BigDecimal::from(0), |acc, s| {
            acc + s.net_pay.clone().unwrap_or_else(|| BigDecimal::from(0))
        });

        if total_net <= BigDecimal::from(0) {
            return Err(AppError::Unprocessable(
                "Nothing to settle — total net is zero".into(),
            ));
        }

        // GL: Dr Payroll payable, Cr Cash/Bank
        let lines = serde_json::json!([
            {
                "account_ref": payload.payroll_payable_uuid,
                "debit": total_net,
                "credit": 0,
                "memo": "Settle net wages payable"
            },
            {
                "account_ref": payload.cash_account_uuid,
                "debit": 0,
                "credit": total_net,
                "memo": "Employee net pay disbursement"
            }
        ]);

        let txn_serial: i64 = sqlx::query_scalar(
            r#"
            SELECT accounting.post_transaction(
                $1, $2, $3, 'payroll_settlement', $4, $5
            )
            "#,
        )
        .bind(format!(
            "Employee settlement payrun {}",
            payrun.serial_id
        ))
        .bind(format!(
            "Settle employees for period {} to {}",
            payrun.pay_period_start, payrun.pay_period_end
        ))
        .bind(user_id)
        .bind(payload.payment_date)
        .bind(&lines)
        .fetch_one(&mut *tx)
        .await?;

        let gl_uuid: Uuid = sqlx::query_scalar(
            "SELECT uuid FROM accounting.transactions WHERE serial_id = $1",
        )
        .bind(txn_serial)
        .fetch_one(&mut *tx)
        .await?;

        let batch = sqlx::query_as::<_, EmployeePaymentBatch>(
            r#"
            INSERT INTO payroll.employee_payment_batches (
                payrun_uuid, payment_reference, payment_date,
                status, gl_transaction_uuid, created_by
            )
            VALUES ($1,$2,$3,'PAID',$4,$5)
            ON CONFLICT (payrun_uuid) DO UPDATE SET
                status = 'PAID',
                gl_transaction_uuid = EXCLUDED.gl_transaction_uuid,
                payment_date = EXCLUDED.payment_date,
                payment_reference = EXCLUDED.payment_reference,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(payload.payrun_uuid)
        .bind(&payload.payment_reference)
        .bind(payload.payment_date)
        .bind(gl_uuid)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        for slip in &slips {
            let amount = slip.net_pay.clone().unwrap_or_else(|| BigDecimal::from(0));
            sqlx::query(
                r#"
                INSERT INTO payroll.employee_payment_lines (
                    payment_batch_uuid, payslip_uuid, amount, status, paid_at
                )
                VALUES ($1,$2,$3,'PAID',NOW())
                ON CONFLICT (payment_batch_uuid, payslip_uuid) DO UPDATE SET
                    status = 'PAID', paid_at = NOW(), amount = EXCLUDED.amount
                "#,
            )
            .bind(batch.uuid)
            .bind(slip.uuid)
            .bind(&amount)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                UPDATE payroll.payslips
                SET status = 'Paid', updated_at = NOW()
                WHERE uuid = $1
                "#,
            )
            .bind(slip.uuid)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(batch)
    }

    async fn get_payment_batch_for_payrun(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
    ) -> Result<Option<EmployeePaymentBatch>, AppError> {
        sqlx::query_as::<_, EmployeePaymentBatch>(
            "SELECT * FROM payroll.employee_payment_batches WHERE payrun_uuid = $1",
        )
        .bind(payrun_uuid)
        .fetch_optional(pool)
        .await
        .map_err(AppError::from)
    }

    async fn create_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Uuid,
        due_date: NaiveDate,
    ) -> Result<Vec<StatutoryRemittance>, AppError> {
        let mut tx = pool.begin().await?;

        let payrun: Payrun = sqlx::query_as(
            "SELECT * FROM payroll.payruns WHERE uuid = $1 FOR UPDATE",
        )
        .bind(payrun_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Payrun not found".into()))?;

        if payrun.status != "Posted" {
            return Err(AppError::Unprocessable(
                "Payrun must be Posted (accrued to GL) before creating statutory remittances".into(),
            ));
        }

        let totals: (BigDecimal, BigDecimal, BigDecimal) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(tax_deducted), 0),
                COALESCE(SUM(social_security), 0),
                COALESCE(SUM(employer_social_security), 0)
            FROM payroll.payslips
            WHERE payrun_uuid = $1
            "#,
        )
        .bind(payrun_uuid)
        .fetch_one(&mut *tx)
        .await?;

        let (paye, nssf_ee, nssf_er) = totals;
        let mut created = Vec::new();

		let code_filter = statutory_code.trim().to_uppercase();
        for (code, amount) in [
            ("PAYE", paye),
            ("NSSF_EE", nssf_ee),
            ("NSSF_ER", nssf_er),
        ] {
            if amount <= BigDecimal::from(0) {
                continue;
            }
			if !code_filter.is_empty()
		        && code_filter != "*"
		        && code_filter != "ALL"
		        && code_filter != code {
		        continue;
		    }
            let row = sqlx::query_as::<_, StatutoryRemittance>(
                r#"
                INSERT INTO payroll.statutory_remittances (
                    payrun_uuid, statutory_code, amount, due_date, status
                )
                VALUES ($1,$2,$3,$4,'PENDING')
                ON CONFLICT (payrun_uuid, statutory_code) DO UPDATE SET
                    amount = EXCLUDED.amount,
                    due_date = EXCLUDED.due_date
                RETURNING *
                "#,
            )
            .bind(payrun_uuid)
            .bind(code)
            .bind(&amount)
            .bind(due_date)
            .fetch_one(&mut *tx)
            .await?;
            created.push(row);
        }

        tx.commit().await?;
        Ok(created)
    }

    async fn remit_statutory(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &RemitStatutory,
    ) -> Result<StatutoryRemittance, AppError> {
        let mut tx = pool.begin().await?;

        let rem: StatutoryRemittance = sqlx::query_as(
            r#"
            SELECT * FROM payroll.statutory_remittances
            WHERE uuid = $1
            FOR UPDATE
            "#,
        )
        .bind(payload.remittance_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Remittance not found".into()))?;

        if rem.status == "PAID" {
            return Ok(rem);
        }
        if rem.status != "PENDING" {
            return Err(AppError::Unprocessable(format!(
                "Cannot remit remittance in status {}",
                rem.status
            )));
        }

        // GL: Dr liability, Cr cash
        let lines = serde_json::json!([
            {
                "account_ref": payload.liability_account_uuid,
                "debit": rem.amount,
                "credit": 0,
                "memo": format!("Remit {}", rem.statutory_code)
            },
            {
                "account_ref": payload.cash_account_uuid,
                "debit": 0,
                "credit": rem.amount,
                "memo": format!("Payment of {}", rem.statutory_code)
            }
        ]);

        let txn_serial: i64 = sqlx::query_scalar(
            r#"
            SELECT accounting.post_transaction(
                $1, $2, $3, 'payroll_remittance', $4, $5
            )
            "#,
        )
        .bind(format!("Remit {}", rem.statutory_code))
        .bind(format!(
            "Statutory remittance {} for payrun",
            rem.statutory_code
        ))
        .bind(user_id)
        .bind(payload.payment_date)
        .bind(&lines)
        .fetch_one(&mut *tx)
        .await?;

        let gl_uuid: Uuid = sqlx::query_scalar(
            "SELECT uuid FROM accounting.transactions WHERE serial_id = $1",
        )
        .bind(txn_serial)
        .fetch_one(&mut *tx)
        .await?;

        let updated = sqlx::query_as::<_, StatutoryRemittance>(
            r#"
            UPDATE payroll.statutory_remittances SET
                status = 'PAID',
                payment_date = $2,
                gl_transaction_uuid = $3
            WHERE uuid = $1
            RETURNING *
            "#,
        )
        .bind(rem.uuid)
        .bind(payload.payment_date)
        .bind(gl_uuid)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    async fn list_statutory_remittances(
        &self,
        pool: &PgPool,
        payrun_uuid: Option<Uuid>,
    ) -> Result<Vec<StatutoryRemittance>, AppError> {
        if let Some(pr) = payrun_uuid {
            sqlx::query_as::<_, StatutoryRemittance>(
                "SELECT * FROM payroll.statutory_remittances WHERE payrun_uuid = $1 ORDER BY statutory_code",
            )
            .bind(pr)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        } else {
            sqlx::query_as::<_, StatutoryRemittance>(
                "SELECT * FROM payroll.statutory_remittances ORDER BY created_at DESC",
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        }
    }
}
