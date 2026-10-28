// src/features/workforce/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::hr_payroll::{
        CompleteChecklistTask, ContractAmendment, CreateContractAmendment, CreateDepartment,
        CreateEmployee, CreateEmploymentContract, CreateJobTitle, CreateLeaveEncashment,
        CreateLeaveRequest, CreateLeaveSchedule, CreateLeaveType, DecideLeaveRequest, Department,
        Employee, EmployeeLifecycle, EmployeePaymentProfile, EmployeeResponse,
        EmployeeStatutoryProfile, EmploymentContract, EmploymentEvent, JobTitle, LeaveBalance,
        LeaveEncashment, LeaveRequest, LeaveSchedule, LeaveType, OnboardingTask, StartExit,
        SupervisorBrief, UpdateEmployee, UpdateLeaveScheduleStatus, UpsertLeaveBalance,
        UpsertPaymentProfile, UpsertStatutoryProfile,
    },
};
use async_trait::async_trait;
use chrono::Datelike;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait HrRepository: Send + Sync {
    // ----- Org -----
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
    async fn list_departments(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Department>, AppError>;
    async fn list_job_titles(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<JobTitle>, AppError>;
    async fn get_department(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<Department, AppError>;
    async fn get_job_title(&self, pool: &PgPool, id: Uuid) -> Result<JobTitle, AppError>;
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
    async fn delete_department(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn delete_job_title(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    // ----- Employees -----
    async fn create_employee(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError>;
    async fn list_employees(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmployeeResponse>, AppError>;
    async fn get_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<EmployeeResponse, AppError>;
    async fn update_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &UpdateEmployee,
    ) -> Result<Employee, AppError>;
    async fn delete_employee(&self, pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError>;

    // ----- Profiles -----
    async fn upsert_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertStatutoryProfile,
    ) -> Result<EmployeeStatutoryProfile, AppError>;
    async fn get_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeStatutoryProfile, AppError>;
    async fn upsert_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertPaymentProfile,
    ) -> Result<EmployeePaymentProfile, AppError>;
    async fn get_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeePaymentProfile, AppError>;
    async fn get_lifecycle(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeLifecycle, AppError>;

    // ----- Contracts -----
    async fn create_contract(
        &self,
        pool: &PgPool,
        payload: &CreateEmploymentContract,
    ) -> Result<EmploymentContract, AppError>;
    async fn list_contracts(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmploymentContract>, AppError>;
    async fn get_contract(&self, pool: &PgPool, id: Uuid) -> Result<EmploymentContract, AppError>;
    async fn add_amendment(
        &self,
        pool: &PgPool,
        contract_uuid: Uuid,
        payload: &CreateContractAmendment,
    ) -> Result<ContractAmendment, AppError>;

    // ----- Leave -----
    async fn create_leave_type(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveType,
    ) -> Result<LeaveType, AppError>;
    async fn list_leave_types(&self, pool: &PgPool) -> Result<Vec<LeaveType>, AppError>;
    async fn upsert_leave_balance(
        &self,
        pool: &PgPool,
        payload: &UpsertLeaveBalance,
    ) -> Result<LeaveBalance, AppError>;
    async fn list_leave_balances(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveBalance>, AppError>;
    async fn create_leave_request(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveRequest,
    ) -> Result<LeaveRequest, AppError>;
    async fn list_leave_requests(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn decide_leave_request(
        &self,
        pool: &PgPool,
        request_id: Uuid,
        payload: &DecideLeaveRequest,
    ) -> Result<LeaveRequest, AppError>;
    async fn create_leave_schedule(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveSchedule,
    ) -> Result<LeaveSchedule, AppError>;
    async fn list_leave_schedules(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveSchedule>, AppError>;
    async fn update_leave_schedule_status(
        &self,
        pool: &PgPool,
        schedule_id: Uuid,
        payload: &UpdateLeaveScheduleStatus,
    ) -> Result<LeaveSchedule, AppError>;
    async fn create_leave_encashment(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveEncashment,
    ) -> Result<LeaveEncashment, AppError>;
    async fn list_leave_encashments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<Vec<LeaveEncashment>, AppError>;

    // ----- Lifecycle ops -----
    async fn list_onboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError>;
    async fn complete_onboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError>;
    async fn complete_onboarding(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError>;
    async fn start_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
        payload: &StartExit,
    ) -> Result<Employee, AppError>;
    async fn list_offboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError>;
    async fn complete_offboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError>;
    async fn complete_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError>;
    async fn list_employment_events(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<EmploymentEvent>, AppError>;
}

pub struct PostgresHrRepo;

impl PostgresHrRepo {
    pub fn new() -> Self {
        Self
    }

    async fn to_employee_response(pool: &PgPool, e: Employee) -> Result<EmployeeResponse, AppError> {
        let department = match e.department_uuid {
            Some(did) => {
                sqlx::query_as::<_, Department>("SELECT * FROM hr.departments WHERE uuid = $1")
                    .bind(did)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };
        let job_title = match e.job_title_uuid {
            Some(jid) => {
                sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles WHERE uuid = $1")
                    .bind(jid)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };
        let supervisor = match e.supervisor_uuid {
            Some(sid) => sqlx::query_as::<_, SupervisorBrief>(
                "SELECT uuid, first_name, last_name FROM hr.employees WHERE uuid = $1",
            )
            .bind(sid)
            .fetch_optional(pool)
            .await?,
            None => None,
        };

        Ok(EmployeeResponse {
            uuid: e.uuid,
            serial_id: e.serial_id,
            first_name: e.first_name,
            last_name: e.last_name,
            middle_name: e.middle_name,
            email: e.email,
            work_email: e.work_email,
            phone_number: e.phone_number,
            date_of_birth: e.date_of_birth,
            gender: e.gender,
            hire_date: e.hire_date,
            department,
            job_title,
            supervisor,
            employment_type: e.employment_type,
            pay_frequency: e.pay_frequency,
            status: e.status,
            party_uuid: e.party_uuid,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
    }
}

#[async_trait]
impl HrRepository for PostgresHrRepo {
    // =========================================================================
    // ORG
    // =========================================================================
    async fn create_department(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        sqlx::query_as::<_, Department>(
            r#"
            INSERT INTO hr.departments (name, description, parent_uuid)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(payload.parent_uuid)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn create_job_title(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        sqlx::query_as::<_, JobTitle>(
            r#"INSERT INTO hr.job_titles (title, description) VALUES ($1, $2) RETURNING *"#,
        )
        .bind(&payload.title)
        .bind(&payload.description)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_departments(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<Department>, AppError> {
        sqlx::query_as::<_, Department>("SELECT * FROM hr.departments ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
    }

    async fn list_job_titles(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<JobTitle>, AppError> {
        sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles ORDER BY title")
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
    }

    async fn get_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<Department, AppError> {
        sqlx::query_as::<_, Department>("SELECT * FROM hr.departments WHERE uuid = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Department not found".into()))
    }

    async fn get_job_title(&self, pool: &PgPool, id: Uuid) -> Result<JobTitle, AppError> {
        sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles WHERE uuid = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Job title not found".into()))
    }

    async fn update_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        sqlx::query_as::<_, Department>(
            r#"
            UPDATE hr.departments
            SET name = $1, description = $2, parent_uuid = COALESCE($3, parent_uuid), updated_at = now()
            WHERE uuid = $4
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(payload.parent_uuid)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Department not found".into()))
    }

    async fn update_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        sqlx::query_as::<_, JobTitle>(
            r#"
            UPDATE hr.job_titles
            SET title = $1, description = $2, updated_at = now()
            WHERE uuid = $3
            RETURNING *
            "#,
        )
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Job title not found".into()))
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
            return Err(AppError::NotFound("Department not found".into()));
        }
        Ok(())
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
            return Err(AppError::NotFound("Job title not found".into()));
        }
        Ok(())
    }

    // =========================================================================
    // EMPLOYEES
    // =========================================================================
    async fn create_employee(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let mut tx = pool.begin().await?;
        let emp: Employee = sqlx::query_as::<_, Employee>(
            r#"
            INSERT INTO hr.employees (
                first_name, last_name, middle_name,
                email, work_email, phone_number,
                date_of_birth, gender, hire_date,
                department_uuid, job_title_uuid, supervisor_uuid,
                employment_type, pay_frequency
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING *
            "#,
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.middle_name)
        .bind(&payload.email)
        .bind(&payload.work_email)
        .bind(&payload.phone_number)
        .bind(payload.date_of_birth)
        .bind(&payload.gender)
        .bind(payload.hire_date)
        .bind(payload.department_uuid)
        .bind(payload.job_title_uuid)
        .bind(payload.supervisor_uuid)
        .bind(&payload.employment_type)
        .bind(&payload.pay_frequency)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("SELECT hr.after_employee_hired($1, $2)")
            .bind(emp.uuid)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(emp)
    }

    async fn list_employees(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<EmployeeResponse>, AppError> {
        let rows: Vec<Employee> =
            sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees ORDER BY last_name, first_name")
                .fetch_all(pool)
                .await?;
        let mut out = Vec::with_capacity(rows.len());
        for e in rows {
            out.push(Self::to_employee_response(pool, e).await?);
        }
        Ok(out)
    }

    async fn get_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
    ) -> Result<EmployeeResponse, AppError> {
        let e: Employee = sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees WHERE uuid = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Employee not found".into()))?;
        Self::to_employee_response(pool, e).await
    }

    async fn update_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        _user_id: Uuid,
        payload: &UpdateEmployee,
    ) -> Result<Employee, AppError> {
        sqlx::query_as::<_, Employee>(
            r#"
            UPDATE hr.employees SET
                first_name = COALESCE($1, first_name),
                last_name = COALESCE($2, last_name),
                middle_name = COALESCE($3, middle_name),
                email = COALESCE($4, email),
                work_email = COALESCE($5, work_email),
                phone_number = COALESCE($6, phone_number),
                date_of_birth = COALESCE($7, date_of_birth),
                gender = COALESCE($8, gender),
                department_uuid = COALESCE($9, department_uuid),
                job_title_uuid = COALESCE($10, job_title_uuid),
                supervisor_uuid = COALESCE($11, supervisor_uuid),
                employment_type = COALESCE($12, employment_type),
                pay_frequency = COALESCE($13, pay_frequency),
                updated_at = now()
            WHERE uuid = $14
            RETURNING *
            "#,
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.middle_name)
        .bind(&payload.email)
        .bind(&payload.work_email)
        .bind(&payload.phone_number)
        .bind(payload.date_of_birth)
        .bind(&payload.gender)
        .bind(payload.department_uuid)
        .bind(payload.job_title_uuid)
        .bind(payload.supervisor_uuid)
        .bind(&payload.employment_type)
        .bind(&payload.pay_frequency)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee not found".into()))
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
            return Err(AppError::NotFound("Employee not found".into()));
        }
        Ok(())
    }

    // =========================================================================
    // PROFILES
    // =========================================================================
    async fn upsert_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertStatutoryProfile,
    ) -> Result<EmployeeStatutoryProfile, AppError> {
        sqlx::query_as::<_, EmployeeStatutoryProfile>(
            r#"
            INSERT INTO hr.employee_statutory_profiles (
                employee_uuid, national_id, tax_id, social_security_number, other_ids
            )
            VALUES ($1, $2, $3, $4, COALESCE($5, '{}'::jsonb))
            ON CONFLICT (employee_uuid) DO UPDATE SET
                national_id = COALESCE(EXCLUDED.national_id, hr.employee_statutory_profiles.national_id),
                tax_id = COALESCE(EXCLUDED.tax_id, hr.employee_statutory_profiles.tax_id),
                social_security_number = COALESCE(EXCLUDED.social_security_number, hr.employee_statutory_profiles.social_security_number),
                other_ids = COALESCE(EXCLUDED.other_ids, hr.employee_statutory_profiles.other_ids),
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(employee_uuid)
        .bind(&payload.national_id)
        .bind(&payload.tax_id)
        .bind(&payload.social_security_number)
        .bind(&payload.other_ids)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn get_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeStatutoryProfile, AppError> {
        sqlx::query_as::<_, EmployeeStatutoryProfile>(
            "SELECT * FROM hr.employee_statutory_profiles WHERE employee_uuid = $1",
        )
        .bind(employee_uuid)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Statutory profile not found".into()))
    }

    async fn upsert_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertPaymentProfile,
    ) -> Result<EmployeePaymentProfile, AppError> {
        sqlx::query_as::<_, EmployeePaymentProfile>(
            r#"
            INSERT INTO hr.employee_payment_profiles (
                employee_uuid, bank_name, bank_account_number, bank_branch,
                payment_method, currency_code
            )
            VALUES (
                $1, $2, $3, $4,
                COALESCE($5, 'Bank Transfer'),
                COALESCE($6, 'UGX')
            )
            ON CONFLICT (employee_uuid) DO UPDATE SET
                bank_name = COALESCE(EXCLUDED.bank_name, hr.employee_payment_profiles.bank_name),
                bank_account_number = COALESCE(EXCLUDED.bank_account_number, hr.employee_payment_profiles.bank_account_number),
                bank_branch = COALESCE(EXCLUDED.bank_branch, hr.employee_payment_profiles.bank_branch),
                payment_method = COALESCE(EXCLUDED.payment_method, hr.employee_payment_profiles.payment_method),
                currency_code = COALESCE(EXCLUDED.currency_code, hr.employee_payment_profiles.currency_code),
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(employee_uuid)
        .bind(&payload.bank_name)
        .bind(&payload.bank_account_number)
        .bind(&payload.bank_branch)
        .bind(&payload.payment_method)
        .bind(&payload.currency_code)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn get_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeePaymentProfile, AppError> {
        sqlx::query_as::<_, EmployeePaymentProfile>(
            "SELECT * FROM hr.employee_payment_profiles WHERE employee_uuid = $1",
        )
        .bind(employee_uuid)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Payment profile not found".into()))
    }

    async fn get_lifecycle(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeLifecycle, AppError> {
        sqlx::query_as::<_, EmployeeLifecycle>(
            "SELECT * FROM hr.employee_lifecycle WHERE employee_uuid = $1",
        )
        .bind(employee_uuid)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Lifecycle record not found".into()))
    }

    // =========================================================================
    // CONTRACTS
    // =========================================================================
    async fn create_contract(
        &self,
        pool: &PgPool,
        payload: &CreateEmploymentContract,
    ) -> Result<EmploymentContract, AppError> {
        sqlx::query_as::<_, EmploymentContract>(
            r#"
            INSERT INTO hr.employment_contracts (
                employee_uuid, contract_code, contract_type, start_date, end_date,
                job_title_uuid, department_uuid, basic_salary, pay_frequency,
                employment_type, probation_end_date, notice_days, status, notes
            )
            VALUES (
                $1, $2, COALESCE($3, 'Permanent'), $4, $5,
                $6, $7, $8, $9, $10, $11, $12, COALESCE($13, 'Active'), $14
            )
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(&payload.contract_code)
        .bind(&payload.contract_type)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(payload.job_title_uuid)
        .bind(payload.department_uuid)
        .bind(&payload.basic_salary)
        .bind(&payload.pay_frequency)
        .bind(&payload.employment_type)
        .bind(payload.probation_end_date)
        .bind(payload.notice_days)
        .bind(&payload.status)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_contracts(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmploymentContract>, AppError> {
        if let Some(eid) = employee_uuid {
            sqlx::query_as::<_, EmploymentContract>(
                "SELECT * FROM hr.employment_contracts WHERE employee_uuid = $1 ORDER BY start_date DESC",
            )
            .bind(eid)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        } else {
            sqlx::query_as::<_, EmploymentContract>(
                "SELECT * FROM hr.employment_contracts ORDER BY start_date DESC",
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        }
    }

    async fn get_contract(&self, pool: &PgPool, id: Uuid) -> Result<EmploymentContract, AppError> {
        sqlx::query_as::<_, EmploymentContract>(
            "SELECT * FROM hr.employment_contracts WHERE uuid = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Contract not found".into()))
    }

    async fn add_amendment(
        &self,
        pool: &PgPool,
        contract_uuid: Uuid,
        payload: &CreateContractAmendment,
    ) -> Result<ContractAmendment, AppError> {
        let mut tx = pool.begin().await?;
        let row: ContractAmendment = sqlx::query_as(
            r#"
            INSERT INTO hr.contract_amendments (
                contract_uuid, effective_date, amendment_type,
                old_value, new_value, basic_salary, job_title_uuid, notes
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(contract_uuid)
        .bind(payload.effective_date)
        .bind(&payload.amendment_type)
        .bind(&payload.old_value)
        .bind(&payload.new_value)
        .bind(&payload.basic_salary)
        .bind(payload.job_title_uuid)
        .bind(&payload.notes)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(ref sal) = payload.basic_salary {
            sqlx::query(
                "UPDATE hr.employment_contracts SET basic_salary = $2, updated_at = now() WHERE uuid = $1",
            )
            .bind(contract_uuid)
            .bind(sal)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(row)
    }

    // =========================================================================
    // LEAVE
    // =========================================================================
    async fn create_leave_type(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveType,
    ) -> Result<LeaveType, AppError> {
        sqlx::query_as::<_, LeaveType>(
            r#"
            INSERT INTO hr.leave_types (
                code, name, is_paid, grant_mode, unused_treatment,
                default_days_per_year, requires_approval
            )
            VALUES (
                $1, $2, COALESCE($3, true), COALESCE($4, 'REQUEST'), COALESCE($5, 'ACCRUE'),
                $6, COALESCE($7, true)
            )
            RETURNING *
            "#,
        )
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(payload.is_paid)
        .bind(&payload.grant_mode)
        .bind(&payload.unused_treatment)
        .bind(&payload.default_days_per_year)
        .bind(payload.requires_approval)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_leave_types(&self, pool: &PgPool) -> Result<Vec<LeaveType>, AppError> {
        sqlx::query_as::<_, LeaveType>(
            "SELECT * FROM hr.leave_types WHERE COALESCE(is_active, true) ORDER BY code",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    async fn upsert_leave_balance(
        &self,
        pool: &PgPool,
        payload: &UpsertLeaveBalance,
    ) -> Result<LeaveBalance, AppError> {
        sqlx::query_as::<_, LeaveBalance>(
            r#"
            INSERT INTO hr.leave_balances (
                employee_uuid, leave_type_uuid, year, entitled, taken, encashed
            )
            VALUES ($1, $2, $3, $4, COALESCE($5, 0), COALESCE($6, 0))
            ON CONFLICT (employee_uuid, leave_type_uuid, year) DO UPDATE SET
                entitled = EXCLUDED.entitled,
                taken = COALESCE(EXCLUDED.taken, hr.leave_balances.taken),
                encashed = COALESCE(EXCLUDED.encashed, hr.leave_balances.encashed),
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.leave_type_uuid)
        .bind(payload.year)
        .bind(&payload.entitled)
        .bind(&payload.taken)
        .bind(&payload.encashed)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_leave_balances(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveBalance>, AppError> {
        let rows = sqlx::query_as::<_, LeaveBalance>(
            "SELECT * FROM hr.leave_balances ORDER BY year DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter(|b| employee_uuid.map(|e| b.employee_uuid == e).unwrap_or(true))
            .filter(|b| year.map(|y| b.year == y).unwrap_or(true))
            .collect())
    }

    async fn create_leave_request(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveRequest,
    ) -> Result<LeaveRequest, AppError> {
        sqlx::query_as::<_, LeaveRequest>(
            r#"
            INSERT INTO hr.leave_requests (
                employee_uuid, leave_type_uuid, start_date, end_date, days, reason
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.leave_type_uuid)
        .bind(payload.start_date)
        .bind(payload.end_date)
        .bind(&payload.days)
        .bind(&payload.reason)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_leave_requests(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        if let Some(eid) = employee_uuid {
            sqlx::query_as::<_, LeaveRequest>(
                "SELECT * FROM hr.leave_requests WHERE employee_uuid = $1 ORDER BY start_date DESC",
            )
            .bind(eid)
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        } else {
            sqlx::query_as::<_, LeaveRequest>(
                "SELECT * FROM hr.leave_requests ORDER BY start_date DESC",
            )
            .fetch_all(pool)
            .await
            .map_err(AppError::from)
        }
    }

    async fn decide_leave_request(
        &self,
        pool: &PgPool,
        request_id: Uuid,
        payload: &DecideLeaveRequest,
    ) -> Result<LeaveRequest, AppError> {
        let mut tx = pool.begin().await?;
        let req: LeaveRequest = sqlx::query_as(
            r#"
            UPDATE hr.leave_requests
            SET status = $2, approver_uuid = $3, decided_at = now()
            WHERE uuid = $1 AND status = 'Pending'
            RETURNING *
            "#,
        )
        .bind(request_id)
        .bind(&payload.status)
        .bind(payload.approver_uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::Unprocessable("Leave request not pending or missing".into()))?;

        if payload.status == "Approved" {
            let year = req.start_date.year();
            sqlx::query(
                r#"
                UPDATE hr.leave_balances
                SET taken = taken + $4, updated_at = now()
                WHERE employee_uuid = $1 AND leave_type_uuid = $2 AND year = $3
                "#,
            )
            .bind(req.employee_uuid)
            .bind(req.leave_type_uuid)
            .bind(year)
            .bind(&req.days)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(req)
    }

    async fn create_leave_schedule(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveSchedule,
    ) -> Result<LeaveSchedule, AppError> {
        sqlx::query_as::<_, LeaveSchedule>(
            r#"
            INSERT INTO hr.leave_schedules (
                employee_uuid, leave_type_uuid, year,
                planned_start, planned_end, days_planned, source, notes
            )
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 'HR'), $8)
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.leave_type_uuid)
        .bind(payload.year)
        .bind(payload.planned_start)
        .bind(payload.planned_end)
        .bind(&payload.days_planned)
        .bind(&payload.source)
        .bind(&payload.notes)
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    }

    async fn list_leave_schedules(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveSchedule>, AppError> {
        let rows = sqlx::query_as::<_, LeaveSchedule>(
            "SELECT * FROM hr.leave_schedules ORDER BY planned_start",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter(|s| employee_uuid.map(|e| s.employee_uuid == e).unwrap_or(true))
            .filter(|s| year.map(|y| s.year == y).unwrap_or(true))
            .collect())
    }

    async fn update_leave_schedule_status(
        &self,
        pool: &PgPool,
        schedule_id: Uuid,
        payload: &UpdateLeaveScheduleStatus,
    ) -> Result<LeaveSchedule, AppError> {
        let mut tx = pool.begin().await?;
        let sched: LeaveSchedule = sqlx::query_as(
            r#"
            UPDATE hr.leave_schedules SET status = $2
            WHERE uuid = $1
            RETURNING *
            "#,
        )
        .bind(schedule_id)
        .bind(&payload.status)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Leave schedule not found".into()))?;

        if payload.status == "Taken" {
            sqlx::query(
                r#"
                UPDATE hr.leave_balances
                SET taken = taken + $4, updated_at = now()
                WHERE employee_uuid = $1 AND leave_type_uuid = $2 AND year = $3
                "#,
            )
            .bind(sched.employee_uuid)
            .bind(sched.leave_type_uuid)
            .bind(sched.year)
            .bind(&sched.days_planned)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(sched)
    }

    async fn create_leave_encashment(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveEncashment,
    ) -> Result<LeaveEncashment, AppError> {
        let mut tx = pool.begin().await?;
        let row: LeaveEncashment = sqlx::query_as(
            r#"
            INSERT INTO hr.leave_encashments (
                employee_uuid, leave_type_uuid, year, days,
                rate_per_day, amount, schedule_uuid, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'Pending')
            RETURNING *
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.leave_type_uuid)
        .bind(payload.year)
        .bind(&payload.days)
        .bind(&payload.rate_per_day)
        .bind(&payload.amount)
        .bind(payload.schedule_uuid)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE hr.leave_balances
            SET encashed = COALESCE(encashed, 0) + $4, updated_at = now()
            WHERE employee_uuid = $1 AND leave_type_uuid = $2 AND year = $3
            "#,
        )
        .bind(payload.employee_uuid)
        .bind(payload.leave_type_uuid)
        .bind(payload.year)
        .bind(&payload.days)
        .execute(&mut *tx)
        .await?;

        if let Some(sid) = payload.schedule_uuid {
            sqlx::query(
                "UPDATE hr.leave_schedules SET status = 'Encashed' WHERE uuid = $1",
            )
            .bind(sid)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(row)
    }

    async fn list_leave_encashments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<Vec<LeaveEncashment>, AppError> {
        let rows = sqlx::query_as::<_, LeaveEncashment>(
            "SELECT * FROM hr.leave_encashments ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter(|e| employee_uuid.map(|id| e.employee_uuid == id).unwrap_or(true))
            .filter(|e| status.map(|s| e.status == s).unwrap_or(true))
            .collect())
    }

    // =========================================================================
    // LIFECYCLE
    // =========================================================================
    async fn list_onboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError> {
        sqlx::query_as::<_, OnboardingTask>(
            r#"
            SELECT * FROM hr.onboarding_tasks
            WHERE employee_uuid = $1
            ORDER BY sort_order, code
            "#,
        )
        .bind(employee_uuid)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    async fn complete_onboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError> {
        sqlx::query_as::<_, OnboardingTask>(
            r#"
            UPDATE hr.onboarding_tasks
            SET is_done = true, done_at = now(), done_by = $2,
                notes = COALESCE($3, notes), updated_at = now()
            WHERE uuid = $1
            RETURNING *
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .bind(&payload.notes)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Onboarding task not found".into()))
    }

    async fn complete_onboarding(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        sqlx::query_as::<_, Employee>("SELECT * FROM hr.complete_onboarding($1, $2)")
            .bind(employee_uuid)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Unprocessable(e.to_string()))
    }

    async fn start_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
        payload: &StartExit,
    ) -> Result<Employee, AppError> {
        sqlx::query_as::<_, Employee>(
            "SELECT * FROM hr.start_exit($1, $2, $3, COALESCE($4, CURRENT_DATE), $5, $6)",
        )
        .bind(employee_uuid)
        .bind(payload.last_working_date)
        .bind(&payload.exit_reason_code)
        .bind(payload.notice_start_date)
        .bind(&payload.notes)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Unprocessable(e.to_string()))
    }

    async fn list_offboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError> {
        sqlx::query_as::<_, OnboardingTask>(
            r#"
            SELECT * FROM hr.offboarding_tasks
            WHERE employee_uuid = $1
            ORDER BY sort_order, code
            "#,
        )
        .bind(employee_uuid)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }

    async fn complete_offboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError> {
        sqlx::query_as::<_, OnboardingTask>(
            r#"
            UPDATE hr.offboarding_tasks
            SET is_done = true, done_at = now(), done_by = $2,
                notes = COALESCE($3, notes), updated_at = now()
            WHERE uuid = $1
            RETURNING *
            "#,
        )
        .bind(task_id)
        .bind(user_id)
        .bind(&payload.notes)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Offboarding task not found".into()))
    }

    async fn complete_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        sqlx::query_as::<_, Employee>("SELECT * FROM hr.complete_exit($1, $2)")
            .bind(employee_uuid)
            .bind(user_id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Unprocessable(e.to_string()))
    }

    async fn list_employment_events(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<EmploymentEvent>, AppError> {
        sqlx::query_as::<_, EmploymentEvent>(
            r#"
            SELECT * FROM hr.employment_events
            WHERE employee_uuid = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(employee_uuid)
        .fetch_all(pool)
        .await
        .map_err(AppError::from)
    }
}


// src/features/hr/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::employee::{
        CreateDepartment, CreateEmployee, CreateJobTitle, Department, Employee, EmployeeResponse,
        JobTitle, SupervisorResponse,
    },
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

    async fn list_employees(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmployeeResponse>, AppError>;
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
    ) -> Result<EmployeeResponse, AppError>;
    async fn get_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Department, AppError>;
    async fn get_job_title(&self, pool: &PgPool, id: Uuid) -> Result<JobTitle, AppError>;

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
    user_id: Uuid,
    payload: &CreateEmployee,
) -> Result<Employee, AppError> {
    let mut tx = pool.begin().await?;

    let emp: Employee = sqlx::query_as::<_, Employee>(
        r#"
        INSERT INTO hr.employees (
            first_name, last_name, middle_name,
            email, work_email, phone_number,
            date_of_birth, gender, hire_date,
            department_uuid, job_title_uuid, supervisor_uuid,
            employment_type, pay_frequency
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
        RETURNING *
        "#,
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.middle_name)
    .bind(&payload.email)
    .bind(&payload.work_email)
    .bind(&payload.phone_number)
    .bind(payload.date_of_birth)
    .bind(&payload.gender)
    .bind(payload.hire_date)
    .bind(payload.department_uuid)
    .bind(payload.job_title_uuid)
    .bind(payload.supervisor_uuid)
    .bind(&payload.employment_type)
    .bind(&payload.pay_frequency)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("SELECT hr.after_employee_hired($1, $2)")
        .bind(emp.uuid)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(emp)
}

async fn update_employee(
    &self,
    pool: &PgPool,
    id: Uuid,
    _user_id: Uuid,
    payload: &UpdateEmployee, // prefer UpdateEmployee over CreateEmployee
) -> Result<Employee, AppError> {
    let emp: Employee = sqlx::query_as::<_, Employee>(
        r#"
        UPDATE hr.employees SET
            first_name      = COALESCE($1, first_name),
            last_name       = COALESCE($2, last_name),
            middle_name     = COALESCE($3, middle_name),
            email           = COALESCE($4, email),
            work_email      = COALESCE($5, work_email),
            phone_number    = COALESCE($6, phone_number),
            date_of_birth   = COALESCE($7, date_of_birth),
            gender          = COALESCE($8, gender),
            department_uuid = COALESCE($9, department_uuid),
            job_title_uuid  = COALESCE($10, job_title_uuid),
            supervisor_uuid = COALESCE($11, supervisor_uuid),
            employment_type = COALESCE($12, employment_type),
            pay_frequency   = COALESCE($13, pay_frequency),
            updated_at      = now()
        WHERE uuid = $14
        RETURNING *
        "#,
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.middle_name)
    .bind(&payload.email)
    .bind(&payload.work_email)
    .bind(&payload.phone_number)
    .bind(payload.date_of_birth)
    .bind(&payload.gender)
    .bind(payload.department_uuid)
    .bind(payload.job_title_uuid)
    .bind(payload.supervisor_uuid)
    .bind(&payload.employment_type)
    .bind(&payload.pay_frequency)
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Employee not found".into()))?;
    Ok(emp)
}

async fn to_employee_response(
    pool: &PgPool,
    e: Employee,
) -> Result<EmployeeResponse, AppError> {
    let department = if let Some(did) = e.department_uuid {
        sqlx::query_as::<_, Department>("SELECT * FROM hr.departments WHERE uuid = $1")
            .bind(did)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let job_title = if let Some(jid) = e.job_title_uuid {
        sqlx::query_as::<_, JobTitle>("SELECT * FROM hr.job_titles WHERE uuid = $1")
            .bind(jid)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    let supervisor = if let Some(sid) = e.supervisor_uuid {
        sqlx::query_as::<_, SupervisorBrief>(
            r#"SELECT uuid, first_name, last_name FROM hr.employees WHERE uuid = $1"#,
        )
        .bind(sid)
        .fetch_optional(pool)
        .await?
    } else {
        None
    };

    Ok(EmployeeResponse {
        uuid: e.uuid,
        serial_id: e.serial_id,
        first_name: e.first_name,
        last_name: e.last_name,
        middle_name: e.middle_name,
        email: e.email,
        work_email: e.work_email,
        phone_number: e.phone_number,
        date_of_birth: e.date_of_birth,
        gender: e.gender,
        hire_date: e.hire_date,
        department,
        job_title,
        supervisor,
        employment_type: e.employment_type,
        pay_frequency: e.pay_frequency,
        status: e.status,
        party_uuid: e.party_uuid,
        created_at: e.created_at,
        updated_at: e.updated_at,
    })
}

    async fn create_employee(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        let emp: Employee = sqlx::query_as::<_, Employee>(
            r#"
            INSERT INTO hr.employees (
                first_name, last_name, email, phone_number, hire_date, termination_date,
                job_title, department_uuid, supervisor_uuid, employment_type, salary,
                pay_frequency
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&payload.phone_number)
        .bind(payload.hire_date)
        .bind(payload.termination_date)
        .bind(payload.job_title)
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
    ) -> Result<Vec<EmployeeResponse>, AppError> {
        let employees: Vec<Employee> =
            sqlx::query_as::<_, Employee>(r#"SELECT * FROM hr.employees"#)
                .fetch_all(pool)
                .await?;

        let mut out: Vec<EmployeeResponse> = Vec::with_capacity(employees.len());

        for employee in employees {
            // job title
            let job_title: Option<JobTitle> = match employee.job_title {
                Some(id) => {
                    sqlx::query_as::<_, JobTitle>(r#"SELECT * FROM hr.job_titles WHERE uuid = $1"#)
                        .bind(id)
                        .fetch_optional(pool)
                        .await?
                }
                None => None,
            };

            // departments
            let department: Option<Department> = match employee.department_uuid {
                Some(id) => {
                    sqlx::query_as::<_, Department>(
                        r#"SELECT * FROM hr.departments WHERE uuid = $1"#,
                    )
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
                }
                None => None,
            };

            // supervisor
            let supervisor: Option<SupervisorResponse> = match employee.supervisor_uuid {
                Some(id) => {
                    sqlx::query_as::<_, SupervisorResponse>(
                        r#"
                        SELECT uuid, first_name, last_name FROM hr.employees WHERE uuid = $1
                    "#,
                    )
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
                }
                None => None,
            };

            out.push(EmployeeResponse {
                uuid: employee.uuid,
                serial_id: employee.serial_id,
                first_name: employee.first_name,
                last_name: employee.last_name,
                email: employee.email,
                phone_number: employee.phone_number,
                hire_date: employee.hire_date,
                termination_date: employee.termination_date,
                job_title,
                department,
                supervisor,
                employment_type: employee.employment_type,
                salary: employee.salary,
                pay_frequency: employee.pay_frequency,
                status: employee.status,
                created_at: employee.created_at,
                updated_at: employee.updated_at,
            });
        }

        Ok(out)
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
    ) -> Result<EmployeeResponse, AppError> {
        let employee: Employee =
            sqlx::query_as::<_, Employee>("SELECT * FROM hr.employees WHERE uuid = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Employee not found".into()))?;

        let job_title: Option<JobTitle> = match employee.job_title {
            Some(id) => {
                sqlx::query_as::<_, JobTitle>(r#"SELECT * FROM hr.job_titles WHERE uuid = $1"#)
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };

        // departments
        let department: Option<Department> = match employee.department_uuid {
            Some(id) => {
                sqlx::query_as::<_, Department>(r#"SELECT * FROM hr.departments WHERE uuid = $1"#)
                    .bind(id)
                    .fetch_optional(pool)
                    .await?
            }
            None => None,
        };

        // supervisor
        let supervisor: Option<SupervisorResponse> = match employee.supervisor_uuid {
            Some(id) => {
                sqlx::query_as::<_, SupervisorResponse>(
                    r#"
                        SELECT uuid, first_name, last_name FROM hr.employees WHERE uuid = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(pool)
                .await?
            }
            None => None,
        };

        Ok(EmployeeResponse {
            uuid: employee.uuid,
            serial_id: employee.serial_id,
            first_name: employee.first_name,
            last_name: employee.last_name,
            email: employee.email,
            phone_number: employee.phone_number,
            hire_date: employee.hire_date,
            termination_date: employee.termination_date,
            job_title,
            department,
            supervisor,
            employment_type: employee.employment_type,
            salary: employee.salary,
            pay_frequency: employee.pay_frequency,
            status: employee.status,
            created_at: employee.created_at,
            updated_at: employee.updated_at,
        })
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

    async fn get_job_title(&self, pool: &PgPool, id: Uuid) -> Result<JobTitle, AppError> {
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
        .bind(payload.job_title)
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
