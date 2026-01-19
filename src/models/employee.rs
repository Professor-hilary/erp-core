// src/models/employee.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Employee {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<Uuid>,
    pub department_uuid: Option<Uuid>,
    pub supervisor_uuid: Option<Uuid>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub job_title: Option<JobTitle>,
    pub department: Option<Department>,
    pub supervisor: Option<SupervisorResponse>,
    pub employment_type: Option<String>,
    pub salary: Option<bigdecimal::BigDecimal>,
    pub pay_frequency: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Department {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartment {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SupervisorResponse {
    pub uuid: Uuid,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct JobTitle {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobTitle {
    pub title: String,
    pub description: Option<String>,
}
