// src/features/payroll/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::payroll::{
        CreatePayComponent, CreatePayslip, CreatePayrun, EligibleEmployeeRow, PayComponent,
        Payrun, Payslip, PostPayrun,
    },
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
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

    async fn process_payrun(&self, pool: &PgPool, payrun_serial_id: i64) -> Result<Payrun, AppError>;

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
        for slip in payslips {
            let tax = slip.tax_deducted.clone().unwrap_or_else(|| BigDecimal::from(0));
            let ss = slip
                .social_security
                .clone()
                .unwrap_or_else(|| BigDecimal::from(0));
            let other = slip
                .other_deductions
                .clone()
                .unwrap_or_else(|| BigDecimal::from(0));
            let er_ss = slip
                .employer_social_security
                .clone()
                .unwrap_or_else(|| BigDecimal::from(0));
            let method = slip
                .payment_method
                .clone()
                .unwrap_or_else(|| "Bank Transfer".into());

            sqlx::query(
                r#"
                INSERT INTO payroll.payslips (
                    payrun_uuid, employee_uuid, gross_pay, basic_pay,
                    tax_deducted, social_security, employer_social_security,
                    other_deductions, payment_method, bank_account
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                "#,
            )
            .bind(payrun_uuid)
            .bind(slip.employee_uuid)
            .bind(&slip.gross_pay)
            .bind(&slip.basic_pay)
            .bind(&tax)
            .bind(&ss)
            .bind(&er_ss)
            .bind(&other)
            .bind(&method)
            .bind(&slip.bank_account)
            .execute(&mut **tx)
            .await?;

            if let Some(items) = &slip.benefits {
                for item in items {
                    sqlx::query(
                        r#"
                        INSERT INTO payroll.payslip_items (
                            payslip_uuid, item_type, description, amount, component_uuid
                        )
                        SELECT uuid, $1, $2, $3, $4
                        FROM payroll.payslips
                        WHERE payrun_uuid = $5 AND employee_uuid = $6
                        ORDER BY uuid DESC
                        LIMIT 1
                        "#,
                    )
                    .bind(&item.item_type)
                    .bind(&item.description)
                    .bind(&item.amount)
                    .bind(item.component_uuid)
                    .bind(payrun_uuid)
                    .bind(slip.employee_uuid)
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
        let payrun = sqlx::query_as::<_, Payrun>(
            r#"
            UPDATE payroll.payruns
            SET status = 'Processed',
                processed_at = now(),
                updated_at = now()
            WHERE serial_id = $1 AND status = 'Pending'
            RETURNING *
            "#,
        )
        .bind(payrun_serial_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            AppError::Unprocessable(
                "Payrun not found or not in Pending status".into(),
            )
        })?;
        Ok(payrun)
    }

    async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError> {
        let labor = payload.labor_expense_id.ok_or_else(|| {
            AppError::Unprocessable("labor_expense_id required until account roles resolve posts".into())
        })?;
        let tax = payload.income_tax_id.ok_or_else(|| {
            AppError::Unprocessable("income_tax_id required until account roles resolve posts".into())
        })?;
        let ss = payload.social_security_id.ok_or_else(|| {
            AppError::Unprocessable("social_security_id required until account roles resolve posts".into())
        })?;
        let cash = payload.cash_account_uuid.ok_or_else(|| {
            AppError::Unprocessable("cash_account_uuid required until account roles resolve posts".into())
        })?;
        let payable = payload.payroll_payable.ok_or_else(|| {
            AppError::Unprocessable("payroll_payable required until account roles resolve posts".into())
        })?;

        // Guard: only Processed
        let status: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM payroll.payruns WHERE serial_id = $1",
        )
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
    }

    async fn list_pay_components(&self, pool: &PgPool) -> Result<Vec<PayComponent>, AppError> {
        sqlx::query_as::<_, PayComponent>(
            "SELECT * FROM payroll.pay_components WHERE COALESCE(is_active, true) ORDER BY code",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
}

/*
// src/features/payroll/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::payrun::{CreatePayrun, CreatePayslip, Payrun, Payslip, PostPayrun},
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait PayrollRepository: Send + Sync {
    // Payroll Section
    async fn create_payrun(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePayrun,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Payrun, AppError>;
    async fn create_payslips(
        &self,
        pool: &PgPool,
        payrun_id: Uuid,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError>;
    async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError>;
    async fn process_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<Payrun, AppError>;
    async fn get_payrun(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Payrun, AppError>;
    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Vec<Payrun>, AppError>;
    async fn get_all_payslips(&self, pool: &PgPool) -> Result<Vec<Payslip>, AppError>;
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
        _pool: &PgPool,
        _user_id: Uuid,
        payload: &CreatePayrun,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Payrun, AppError> {
        let payrun: Payrun = sqlx::query_as::<_, Payrun>(
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
        _pool: &PgPool,
        payrun_uuid: Uuid,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError> {
        for slip in payslips {
            let tax: BigDecimal = slip.tax_deducted.clone().unwrap_or_default();
            let social_security: BigDecimal = slip.social_security.clone().unwrap_or_default();
            let other: BigDecimal = slip.other_deductions.clone().unwrap_or_default();
            let method: String = slip
                .payment_method
                .clone()
                .unwrap_or("Bank Transfer".to_string());

            let _ = sqlx::query(
                r#"
                INSERT INTO payroll.payslips (payrun_uuid, employee_uuid, gross_pay, tax_deducted, social_security, other_deductions, payment_method, bank_account)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#
            )
            .bind(payrun_uuid)
            .bind(slip.employee_uuid)
            .bind(&slip.gross_pay)
            .bind(tax)
            .bind(social_security)
            .bind(other)
            .bind(method)
            .bind(&slip.bank_account)
            .execute(&mut **tx)
            .await?;

            if let Some(items) = &slip.benefits {
                for item in items {
                    // Get values to insert into payslip items table from payslips table given the payrun uuid
                    sqlx::query(
                        r#"
                            INSERT INTO payroll.payslip_items (payslip_uuid, item_type, description, amount)
                            SELECT uuid, $1, $2, $3 FROM payroll.payslips WHERE payrun_uuid = $4
                            AND employee_uuid = $5 ORDER BY uuid DESC LIMIT 1
                        "#,
                    )
                    .bind(&item.item_type)
                    .bind(&item.description)
                    .bind(&item.amount)
                    .bind(payrun_uuid)
                    .bind(slip.employee_uuid)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        Ok(())
    }

    async fn post_payrun(
        &self,
        user_id: Uuid,
        pool: &PgPool,
        payload: &PostPayrun,
    ) -> Result<(), AppError> {
        sqlx::query("SELECT payroll.post_payrun($1, $2, $3, $4, $5, $6, $7)")
            .bind(payload.payrun_serial_id)
            .bind(payload.labor_expense_id)
            .bind(payload.income_tax_id)
            .bind(payload.social_security_id)
            .bind(payload.cash_account_uuid)
            .bind(user_id)
            .bind(payload.payroll_payable)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn process_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<Payrun, AppError> {
        let payrun: Payrun = sqlx::query_as::<_, Payrun>(
            r#"UPDATE payroll.payruns SET status='Processed' WHERE serial_id=$1 RETURNING *"#,
        )
        .bind(payrun_id)
        .fetch_one(pool)
        .await?;
        Ok(payrun)
    }

    async fn get_payrun(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<Payrun, AppError> {
        let payrun: Payrun =
            sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns WHERE uuid = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Payrun not found".into()))?;
        Ok(payrun)
    }

    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Vec<Payrun>, AppError> {
        let payruns = sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns")
            .fetch_all(pool)
            .await
            .map_err(|_| AppError::NotFound("Payrun not found".into()))?;
        Ok(payruns)
    }

    async fn get_all_payslips(&self, pool: &PgPool) -> Result<Vec<Payslip>, AppError> {
        let payslips = sqlx::query_as::<_, Payslip>("SELECT * FROM payroll.payslips")
            .fetch_all(pool)
            .await
            .map_err(|_| AppError::NotFound("Payrun not found".into()))?;
        Ok(payslips)
    }
}
*/
