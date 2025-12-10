// src/features/hr/repository.rs
use crate::models::employee::{
    CreateDepartment, CreateEmployee, CreateJobTitle, Department, Employee, JobTitle,
};
use crate::{
    infrastructure::errors::AppError,
    models::payrun::{CreatePayrun, CreatePayslip, Payrun},
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait HrRepository: Send + Sync {
    // Human Resource Section
    async fn create_employee(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError>;
    async fn create_department(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError>;
    async fn create_job_title(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError>;

    async fn list_employees(&self, pool: &PgPool, user_id: Uuid)
    -> Result<Vec<Employee>, AppError>;
    async fn list_departments(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Department>, AppError>;
    async fn list_job_titles(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JobTitle>, AppError>;

    async fn get_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError>;
    async fn get_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Department, AppError>;
    async fn get_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<JobTitle, AppError>;

    async fn update_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError>;
    async fn update_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError>;
    async fn update_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError>;

    async fn delete_employee(&self, pool: &PgPool, id: Uuid, user_id: Uuid)
    -> Result<(), AppError>;
    async fn delete_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
    async fn delete_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

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
    async fn post_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<(), AppError>;
    async fn process_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<Payrun, AppError>;
    async fn get_payrun(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Payrun, AppError>;
    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Payrun, AppError>;
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
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let emp: Employee = sqlx::query_as::<_, Employee>(
            r#"
            INSERT INTO hr.employees (first_name, last_name, email, phone_number, hire_date, termination_date, job_title, department_uuid, supervisor_uuid, employment_type, salary, pay_frequency)
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
        .bind(payload.department_uuid)
        .bind(payload.supervisor_uuid)
        .bind(&payload.employment_type)
        .bind(&payload.salary)
        .bind(&payload.pay_frequency)
        .fetch_one(pool)
        .await?;
        Ok(emp)
    }

    async fn create_department(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        let department: Department = sqlx::query_as::<_, Department>(
            r#"INSERT INTO hr.departments (name, description)VALUES ($1, $2) RETURNING *"#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .fetch_one(pool)
        .await?;
        Ok(department)
    }

    async fn create_job_title(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        let jobtitle: JobTitle = sqlx::query_as::<_, JobTitle>(
            r#"INSERT INTO hr.job_titles (title, description)VALUES ($1, $2) RETURNING *"#,
        )
        .bind(&payload.title)
        .bind(&payload.description)
        .fetch_one(pool)
        .await?;
        Ok(jobtitle)
    }

    async fn list_employees(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<Employee>, AppError> {
        let rows: Vec<Employee> = sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn list_departments(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<Department>, AppError> {
        let rows: Vec<Department> = sqlx::query_as::<_, Department>("SELECT * FROM hr.departments")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn list_job_titles(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<JobTitle>, AppError> {
        let rows: Vec<JobTitle> = sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn get_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<Employee, AppError> {
        let emp: Employee =
            sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees WHERE uuid = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Employee not found".into()))?;
        Ok(emp)
    }

    async fn get_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<Department, AppError> {
        let emp: Department =
            sqlx::query_as::<_, Department>("SELECT * FROM hr.departments WHERE uuid = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Department not found".into()))?;
        Ok(emp)
    }

    async fn get_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<JobTitle, AppError> {
        let emp: JobTitle =
            sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles WHERE uuid = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("JobTitle not found".into()))?;
        Ok(emp)
    }

    async fn update_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let emp: Employee = sqlx::query_as::<_, Employee>(
            r#"
            UPDATE hr.employees
            SET first_name=COALESCE($1, first_name), last_name=COALESCE($2, last_name), email=COALESCE($3,email), phone_number=COALESCE($4, phone_number), hire_date=COALESCE($5, hire_date), termination_date=COALESCE($6, termination_date),
                job_title=COALESCE($7, job_title), department_uuid=COALESCE($8, department_uuid), supervisor_uuid=COALESCE($9, supervisor_uuid), employment_type=COALESCE($10, employment_type), salary=COALESCE($11,salary), pay_frequency=COALESCE($12, pay_frequency),
                updated_at=now()
            WHERE uuid=$13
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
        .bind(payload.department_uuid)
        .bind(payload.supervisor_uuid)
        .bind(&payload.employment_type)
        .bind(&payload.salary)
        .bind(&payload.pay_frequency)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(emp)
    }

    async fn update_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        let department= sqlx::query_as::<_, Department>(
            r#"UPDATE hr.departments SET name=$1, description=$2, updated_at=now() WHERE uuid=$3 RETURNING *"#
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(department)
    }

    async fn update_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        let department: JobTitle = sqlx::query_as::<_, JobTitle>(
            r#"UPDATE hr.job_titles SET title=$1, description=$2, updated_at=now() WHERE uuid=$3 RETURNING *"#
        )
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(department)
    }

    async fn delete_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM hr.employees WHERE uuid = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Employee not found".into()))
        } else {
            Ok(())
        }
    }

    async fn delete_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM hr.departments WHERE uuid = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Department not found".into()))
        } else {
            Ok(())
        }
    }
    async fn delete_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM hr.job_titles WHERE uuid = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Job title not found".into()))
        } else {
            Ok(())
        }
    }

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
                    sqlx::query(
                        r#"
                            INSERT INTO payroll.payslip_items (uuid, item_type, description, amount)
                            SELECT uuid, $1, $2, $3 FROM payroll.payslips WHERE payrun_uuid = $4
                            AND employee_id = $5 ORDER BY payslip_id DESC LIMIT 1
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

    async fn post_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<(), AppError> {
        sqlx::query("SELECT payroll.post_payrun($1)")
            .bind(payrun_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn process_payrun(&self, pool: &PgPool, payrun_id: i64) -> Result<Payrun, AppError> {
        let payrun: Payrun = sqlx::query_as::<_, Payrun>(
            r#"UPDATE hr.payruns SET status='Processed' WHERE uuid=$1 RETURNING *"#,
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

    async fn get_all_payruns(&self, pool: &PgPool) -> Result<Payrun, AppError> {
        let payrun: Payrun = sqlx::query_as::<_, Payrun>("SELECT * FROM payroll.payruns")
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Payrun not found".into()))?;
        Ok(payrun)
    }
}
