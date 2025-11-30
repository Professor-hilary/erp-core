// src/features/hr/repository.rs
use crate::{
    infrastructure::errors::AppError,
    models::{
        employee::{CreateEmployee, Employee},
        payrun::{CreatePayrun, CreatePayslip, Payrun},
    },
};
use async_trait::async_trait;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait HrRepository: Send + Sync {
    async fn create_employee(
        &self, pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError>;
    async fn list_employees(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Employee>, AppError>;
    async fn get_employee(&self, pool: &PgPool, id: i32, user_id: Uuid) -> Result<Employee, AppError>;
    async fn update_employee(
        &self, pool: &PgPool,
        id: i32,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError>;
    async fn delete_employee(&self, pool: &PgPool, id: i32, user_id: Uuid) -> Result<(), AppError>;

    async fn create_payrun(
        &self, pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePayrun,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<Payrun, AppError>;
    async fn create_payslips(
        &self, pool: &PgPool,
        payrun_id: i32,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError>;
    async fn post_payrun(&self, pool: &PgPool, payrun_id: i32) -> Result<(), AppError>;
    async fn get_payrun(&self, pool: &PgPool, id: i32, user_id: Uuid) -> Result<Payrun, AppError>;
    // Add list_payruns if needed
}

pub struct PostgresHrRepo;

impl PostgresHrRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HrRepository for PostgresHrRepo {
    async fn create_employee(
        &self,pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let emp = sqlx::query_as::<_, Employee>(
            r#"
            INSERT INTO hr.employees (first_name, last_name, email, phone_number, hire_date, termination_date, job_title, department_id, supervisor_id, employment_type, salary, pay_frequency)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&payload.phone_number)
        .bind(payload.hire_date)
        .bind(payload.termination_date)
        .bind(&payload.job_title)
        .bind(payload.department_id)
        .bind(payload.supervisor_id)
        .bind(&payload.employment_type)
        .bind(&payload.salary)
        .bind(&payload.pay_frequency)
        .fetch_one(pool)
        .await?;
        Ok(emp)
    }

    async fn list_employees(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Employee>, AppError> {
        let rows = sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn get_employee(&self, pool: &PgPool, id: i32, _user_id: Uuid) -> Result<Employee, AppError> {
        let emp =
            sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees WHERE employee_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Employee not found".into()))?;
        Ok(emp)
    }

    async fn update_employee(
        &self, pool: &PgPool,
        id: i32,
        _user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let emp = sqlx::query_as::<_, Employee>(
            r#"
            UPDATE hr.employees
            SET first_name=$1, last_name=$2, email=$3, phone_number=$4, hire_date=$5, termination_date=$6,
                job_title=$7, department_id=$8, supervisor_id=$9, employment_type=$10, salary=$11, pay_frequency=$12,
                updated_at=now()
            WHERE employee_id=$13
            RETURNING *
            "#
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&payload.phone_number)
        .bind(payload.hire_date)
        .bind(payload.termination_date)
        .bind(&payload.job_title)
        .bind(payload.department_id)
        .bind(payload.supervisor_id)
        .bind(&payload.employment_type)
        .bind(&payload.salary)
        .bind(&payload.pay_frequency)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(emp)
    }

    async fn delete_employee(&self, pool: &PgPool, id: i32, _user_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM hr.employees WHERE employee_id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Employee not found".into()))
        } else {
            Ok(())
        }
    }

    async fn create_payrun(
        &self, _pool: &PgPool,
        _user_id: Uuid,
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
        &self, _pool: &PgPool,
        payrun_id: i32,
        payslips: &[CreatePayslip],
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), AppError> {
        for slip in payslips {
            let tax = slip.tax_deducted.clone().unwrap_or_default();
            let nssf = slip.nssf.clone().unwrap_or_default();
            let other = slip.other_deductions.clone().unwrap_or_default();
            let method = slip
                .payment_method
                .clone()
                .unwrap_or("Bank Transfer".to_string());

            let _ = sqlx::query(
                r#"
                INSERT INTO payroll.payslips (payrun_id, employee_id, gross_pay, tax_deducted, nssf, other_deductions, payment_method, bank_account)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#
            )
            .bind(payrun_id)
            .bind(slip.employee_id)
            .bind(&slip.gross_pay)
            .bind(tax)
            .bind(nssf)
            .bind(other)
            .bind(method)
            .bind(&slip.bank_account)
            .execute(&mut **tx)
            .await?;

            if let Some(items) = &slip.items {
                for item in items {
                    sqlx::query(
                        r#"
                        INSERT INTO payroll.payslip_items (payslip_id, item_type, description, amount)
                        SELECT payslip_id, $1, $2, $3 FROM payroll.payslips WHERE payrun_id = $4 AND employee_id = $5 ORDER BY payslip_id DESC LIMIT 1
                        "#
                    )
                    .bind(&item.item_type)
                    .bind(&item.description)
                    .bind(&item.amount)
                    .bind(payrun_id)
                    .bind(slip.employee_id)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        Ok(())
    }

    async fn post_payrun(&self, pool: &PgPool, payrun_id: i32) -> Result<(), AppError> {
        sqlx::query("SELECT payroll.post_payrun($1)")
            .bind(payrun_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn get_payrun(&self, pool: &PgPool, id: i32, _user_id: Uuid) -> Result<Payrun, AppError> {
        let payrun =
            sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns WHERE payrun_id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Payrun not found".into()))?;
        Ok(payrun)
    }
}
