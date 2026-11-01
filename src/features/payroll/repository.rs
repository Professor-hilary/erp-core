// src/features/payroll/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::payroll::{
        ComponentAssignmentRow, CreatePayComponent, CreatePayrun, CreatePayslip,
        EligibleEmployeeRow, EmployeePayrollContext, EmployeeTaxPeriodRow, LeaveEncashmentRow,
        NssfRuleRow, PayComponent, Payrun, Payslip, PostPayrun, StatutoryBandRow,
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
                    pp.payment_method,
                    pp.bank_account_number
                FROM hr.employees e
                LEFT JOIN hr.employee_statutory_profiles sp ON sp.employee_uuid = e.uuid
                LEFT JOIN hr.employment_contracts c
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

            let basic_salary = ctx
                .basic_salary
                .clone()
                .unwrap_or_else(|| BigDecimal::from(0));
            let contract_start = ctx.contract_start.unwrap_or(period_start);
            let contract_end = ctx.contract_end;

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

            // Leave encashment as taxable earning
            for (idx, enc) in encashments.iter().enumerate() {
                components.push(ComponentInput {
                    component_uuid: None,
                    code: format!("LEAVE_ENCASH_{}", idx + 1),
                    name: format!(
                        "Leave encashment{}",
                        enc.leave_type_code
                            .as_ref()
                            .map(|c| format!(" ({c})"))
                            .unwrap_or_default()
                    ),
                    kind: ComponentKind::Earning,
                    method: CalculationMethod::Fixed,
                    amount: enc.amount.clone(),
                    rate: None,
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

            let tax_bands: Vec<TaxBand> = bands
                .into_iter()
                .map(|b| TaxBand {
                    lower_bound: b.lower_bound,
                    upper_bound: b.upper_bound,
                    base_tax: b.base_tax,
                    marginal_rate: b.marginal_rate,
                })
                .collect();

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

            let (nssf_employee_rate, nssf_employer_rate, nssf_min_age, nssf_max_age) =
                match nssf {
                    Some(n) => (
                        n.employee_rate,
                        n.employer_rate,
                        n.minimum_age.map(|a| a as u8),
                        n.maximum_age.map(|a| a as u8),
                    ),
                    None => (
                        BigDecimal::from(5) / BigDecimal::from(100),
                        BigDecimal::from(10) / BigDecimal::from(100),
                        Some(16),
                        Some(55),
                    ),
                };

            // 8. Build calculation input and run the engine
            let input = PayrollInput {
                employee_uuid: slip.employee_uuid,
                basic_salary,
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
                nssf_employee_rate,
                nssf_employer_rate,
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
            //    Compute tax on (prior taxable + this taxable), subtract prior withheld.
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

            let (prior_taxable, prior_withheld, tax_period_uuid) = match &prior {
                Some(p) => (p.taxable_income.clone(), p.paye_withheld.clone(), Some(p.uuid)),
                None => (BigDecimal::from(0), BigDecimal::from(0), None),
            };

            let cumulative_taxable = &prior_taxable + &result.taxable_earnings;
            let cumulative_paye = crate::features::payroll::calculation::progressive_tax(
                &cumulative_taxable,
                &input.tax_bands,
            )
            .map_err(|e| AppError::Unprocessable(format!("Cumulative PAYE failed: {e}")))?;

            let period_paye = (&cumulative_paye - &prior_withheld).max(BigDecimal::from(0));
            result.paye = period_paye.clone();
            // Recompute net after PAYE adjustment
            result.net_pay = (&result.gross_pay
                - &result.paye
                - &result.employee_nssf
                - &result.other_deductions)
                .max(BigDecimal::from(0));

            // 10. Persist calculation results onto the payslip
            let snapshot = json!({
                "version": calculation_version,
                "basic_pay": result.basic_pay,
                "gross_pay": result.gross_pay,
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
            .bind(&result.gross_pay) // approx base
            .bind(&nssf_employee_rate)
            .bind(&result.employee_nssf)
            .bind(&nssf_employer_rate)
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
            SELECT payroll.post_payrun($1,$2,$3,$4,$5,$6,$7,$8)
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
}
