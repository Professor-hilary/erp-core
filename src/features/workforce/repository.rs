// src/features/hr/repository.rs
use crate::infrastructure::errors::AppError;
use crate::models::employee::{
    CreateDepartment, CreateEmployee, CreateJobTitle, Department, Employee, JobTitle,
};
use async_trait::async_trait;
use sqlx::PgPool;
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
}
