// src/features/workforce/services.rs
use crate::{
    features::workforce::repository::HrRepository,
    interface::api::errors::AppError,
    models::employee::*,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct HrService<R: HrRepository> {
    repo: R,
}

impl<R: HrRepository> HrService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // ----- Org -----
    pub async fn create_department(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        self.repo.create_department(pool, user_id, payload).await
    }

    pub async fn create_job_title(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        self.repo.create_job_title(pool, user_id, payload).await
    }

    pub async fn list_departments(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Department>, AppError> {
        self.repo.list_departments(pool, user_id).await
    }

    pub async fn list_job_titles(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JobTitle>, AppError> {
        self.repo.list_job_titles(pool, user_id).await
    }

    pub async fn get_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Department, AppError> {
        self.repo.get_department(pool, id, user_id).await
    }

    pub async fn get_job_title(&self, pool: &PgPool, id: Uuid) -> Result<JobTitle, AppError> {
        self.repo.get_job_title(pool, id).await
    }

    pub async fn update_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateDepartment,
    ) -> Result<Department, AppError> {
        self.repo
            .update_department(pool, id, user_id, payload)
            .await
    }

    pub async fn update_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &CreateJobTitle,
    ) -> Result<JobTitle, AppError> {
        self.repo
            .update_job_title(pool, id, user_id, payload)
            .await
    }

    pub async fn delete_department(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_department(pool, id, user_id).await
    }

    pub async fn delete_job_title(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_job_title(pool, id, user_id).await
    }

    // ----- Employees -----
    pub async fn create_employee(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateEmployee,
    ) -> Result<Employee, AppError> {
        self.repo.create_employee(pool, user_id, payload).await
    }

    pub async fn list_employees(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<EmployeeResponse>, AppError> {
        self.repo.list_employees(pool, user_id).await
    }

    pub async fn get_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<EmployeeResponse, AppError> {
        self.repo.get_employee(pool, id, user_id).await
    }

    pub async fn update_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        payload: &UpdateEmployee,
    ) -> Result<Employee, AppError> {
        self.repo.update_employee(pool, id, user_id, payload).await
    }

    pub async fn delete_employee(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_employee(pool, id, user_id).await
    }

    // ----- Profiles -----
    pub async fn upsert_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertStatutoryProfile,
    ) -> Result<EmployeeStatutoryProfile, AppError> {
        self.repo
            .upsert_statutory_profile(pool, employee_uuid, payload)
            .await
    }

    pub async fn get_statutory_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeStatutoryProfile, AppError> {
        self.repo.get_statutory_profile(pool, employee_uuid).await
    }

    pub async fn upsert_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        payload: &UpsertPaymentProfile,
    ) -> Result<EmployeePaymentProfile, AppError> {
        self.repo
            .upsert_payment_profile(pool, employee_uuid, payload)
            .await
    }

    pub async fn get_payment_profile(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeePaymentProfile, AppError> {
        self.repo.get_payment_profile(pool, employee_uuid).await
    }

    pub async fn get_lifecycle(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<EmployeeLifecycle, AppError> {
        self.repo.get_lifecycle(pool, employee_uuid).await
    }

    // ----- Contracts -----
    pub async fn create_contract(
        &self,
        pool: &PgPool,
        payload: &CreateEmploymentContract,
    ) -> Result<EmploymentContract, AppError> {
        self.repo.create_contract(pool, payload).await
    }

    pub async fn list_contracts(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<EmploymentContract>, AppError> {
        self.repo.list_contracts(pool, employee_uuid).await
    }

    pub async fn get_contract(
        &self,
        pool: &PgPool,
        id: Uuid,
    ) -> Result<EmploymentContract, AppError> {
        self.repo.get_contract(pool, id).await
    }

    pub async fn add_amendment(
        &self,
        pool: &PgPool,
        contract_uuid: Uuid,
        payload: &CreateContractAmendment,
    ) -> Result<ContractAmendment, AppError> {
        self.repo.add_amendment(pool, contract_uuid, payload).await
    }

    // ----- Leave -----
    pub async fn create_leave_type(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveType,
    ) -> Result<LeaveType, AppError> {
        self.repo.create_leave_type(pool, payload).await
    }

    pub async fn list_leave_types(&self, pool: &PgPool) -> Result<Vec<LeaveType>, AppError> {
        self.repo.list_leave_types(pool).await
    }

    pub async fn upsert_leave_balance(
        &self,
        pool: &PgPool,
        payload: &UpsertLeaveBalance,
    ) -> Result<LeaveBalance, AppError> {
        self.repo.upsert_leave_balance(pool, payload).await
    }

    pub async fn list_leave_balances(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveBalance>, AppError> {
        self.repo
            .list_leave_balances(pool, employee_uuid, year)
            .await
    }

    pub async fn create_leave_request(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveRequest,
    ) -> Result<LeaveRequest, AppError> {
        self.repo.create_leave_request(pool, payload).await
    }

    pub async fn list_leave_requests(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        self.repo.list_leave_requests(pool, employee_uuid).await
    }

    pub async fn decide_leave_request(
        &self,
        pool: &PgPool,
        request_id: Uuid,
        payload: &DecideLeaveRequest,
    ) -> Result<LeaveRequest, AppError> {
        self.repo
            .decide_leave_request(pool, request_id, payload)
            .await
    }

    pub async fn create_leave_schedule(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveSchedule,
    ) -> Result<LeaveSchedule, AppError> {
        self.repo.create_leave_schedule(pool, payload).await
    }

    pub async fn list_leave_schedules(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        year: Option<i32>,
    ) -> Result<Vec<LeaveSchedule>, AppError> {
        self.repo
            .list_leave_schedules(pool, employee_uuid, year)
            .await
    }

    pub async fn update_leave_schedule_status(
        &self,
        pool: &PgPool,
        schedule_id: Uuid,
        payload: &UpdateLeaveScheduleStatus,
    ) -> Result<LeaveSchedule, AppError> {
        self.repo
            .update_leave_schedule_status(pool, schedule_id, payload)
            .await
    }

    pub async fn create_leave_encashment(
        &self,
        pool: &PgPool,
        payload: &CreateLeaveEncashment,
    ) -> Result<LeaveEncashment, AppError> {
        self.repo.create_leave_encashment(pool, payload).await
    }

    pub async fn list_leave_encashments(
        &self,
        pool: &PgPool,
        employee_uuid: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<Vec<LeaveEncashment>, AppError> {
        self.repo
            .list_leave_encashments(pool, employee_uuid, status)
            .await
    }

    // ----- Lifecycle -----
    pub async fn list_onboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError> {
        self.repo.list_onboarding_tasks(pool, employee_uuid).await
    }

    pub async fn complete_onboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError> {
        self.repo
            .complete_onboarding_task(pool, task_id, user_id, payload)
            .await
    }

    pub async fn complete_onboarding(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        self.repo
            .complete_onboarding(pool, employee_uuid, user_id)
            .await
    }

    pub async fn start_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
        payload: &StartExit,
    ) -> Result<Employee, AppError> {
        self.repo
            .start_exit(pool, employee_uuid, user_id, payload)
            .await
    }

    pub async fn list_offboarding_tasks(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<OnboardingTask>, AppError> {
        self.repo
            .list_offboarding_tasks(pool, employee_uuid)
            .await
    }

    pub async fn complete_offboarding_task(
        &self,
        pool: &PgPool,
        task_id: Uuid,
        user_id: Uuid,
        payload: &CompleteChecklistTask,
    ) -> Result<OnboardingTask, AppError> {
        self.repo
            .complete_offboarding_task(pool, task_id, user_id, payload)
            .await
    }

    pub async fn complete_exit(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Employee, AppError> {
        self.repo.complete_exit(pool, employee_uuid, user_id).await
    }

    pub async fn list_employment_events(
        &self,
        pool: &PgPool,
        employee_uuid: Uuid,
    ) -> Result<Vec<EmploymentEvent>, AppError> {
        self.repo
            .list_employment_events(pool, employee_uuid)
            .await
    }
}

