// src/features/payroll/repository.rs
use crate::{
    infrastructure::errors::AppError,
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
            .bind(&payload.payrun_serial_id)
            .bind(&payload.labor_expense_id)
            .bind(&payload.income_tax_id)
            .bind(&payload.social_security_id)
            .bind(&payload.cash_account_uuid)
            .bind(user_id)
            .bind(&payload.payroll_payable)
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
