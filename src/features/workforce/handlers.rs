// src/features/workforce/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, Query, State},
    response::Response,
    routing::{delete, get, patch, post, put},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::workforce::{repository::PostgresHrRepo, services::HrService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::hr_payroll::{
        CompleteChecklistTask, CreateContractAmendment, CreateDepartment, CreateEmployee,
        CreateEmploymentContract, CreateJobTitle, CreateLeaveEncashment, CreateLeaveRequest,
        CreateLeaveSchedule, CreateLeaveType, DecideLeaveRequest, StartExit, UpdateEmployee,
        UpdateLeaveScheduleStatus, UpsertLeaveBalance, UpsertPaymentProfile, UpsertStatutoryProfile,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Org
        .route("/create/department", post(create_department))
        .route("/create/jobtitle", post(create_job_title))
        .route("/list/departments", get(list_departments))
        .route("/list/jobtitles", get(list_job_titles))
        .route("/get/department/{id}", get(get_department))
        .route("/get/jobtitle/{id}", get(get_job_title))
        .route("/update/department/{id}", patch(update_department))
        .route("/update/jobtitle/{id}", patch(update_job_title))
        .route("/delete/department/{id}", delete(delete_department))
        .route("/delete/jobtitle/{id}", delete(delete_job_title))
        // Employees
        .route("/create/employee", post(create_employee))
        .route("/list/employees", get(list_employees))
        .route("/get/employee/{id}", get(get_employee))
        .route("/update/employee/{id}", patch(update_employee))
        .route("/delete/employee/{id}", delete(delete_employee))
        // Profiles
        .route(
            "/employees/{id}/statutory-profile",
            put(upsert_statutory_profile).get(get_statutory_profile),
        )
        .route(
            "/employees/{id}/payment-profile",
            put(upsert_payment_profile).get(get_payment_profile),
        )
        .route("/employees/{id}/lifecycle", get(get_lifecycle))
        .route("/employees/{id}/events", get(list_employment_events))
        // Contracts
        .route("/contracts", post(create_contract).get(list_contracts))
        .route("/contracts/{id}", get(get_contract))
        .route("/contracts/{id}/amendments", post(add_amendment))
        // Leave
        .route("/leave-types", post(create_leave_type).get(list_leave_types))
        .route(
            "/leave-balances",
            post(upsert_leave_balance).get(list_leave_balances),
        )
        .route(
            "/leave-requests",
            post(create_leave_request).get(list_leave_requests),
        )
        .route("/leave-requests/{id}/decide", post(decide_leave_request))
        .route(
            "/leave-schedules",
            post(create_leave_schedule).get(list_leave_schedules),
        )
        .route(
            "/leave-schedules/{id}/status",
            post(update_leave_schedule_status),
        )
        .route(
            "/leave-encashments",
            post(create_leave_encashment).get(list_leave_encashments),
        )
        // Lifecycle
        .route(
            "/employees/{id}/onboarding-tasks",
            get(list_onboarding_tasks),
        )
        .route(
            "/onboarding-tasks/{id}/complete",
            post(complete_onboarding_task),
        )
        .route(
            "/employees/{id}/complete-onboarding",
            post(complete_onboarding),
        )
        .route("/employees/{id}/start-exit", post(start_exit))
        .route(
            "/employees/{id}/offboarding-tasks",
            get(list_offboarding_tasks),
        )
        .route(
            "/offboarding-tasks/{id}/complete",
            post(complete_offboarding_task),
        )
        .route("/employees/{id}/complete-exit", post(complete_exit))
}

fn svc() -> HrService<PostgresHrRepo> {
    HrService::new(PostgresHrRepo::new())
}

// ----- Query helpers -----
#[derive(Debug, Deserialize)]
pub struct EmployeeFilter {
    pub employee_uuid: Option<Uuid>,
    pub year: Option<i32>,
    pub status: Option<String>,
}

// =============================================================================
// ORG
// =============================================================================
async fn create_department(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDepartment>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_department(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Department created"))
}

async fn create_job_title(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateJobTitle>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_job_title(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Job title created"))
}

async fn list_departments(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_departments(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(rows, "Departments fetched"))
}

async fn list_job_titles(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_job_titles(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(rows, "Job titles fetched"))
}

async fn get_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .get_department(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(row, "Department fetched"))
}

async fn get_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc().get_job_title(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(row, "Job title fetched"))
}

async fn update_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDepartment>,
) -> Result<Response, AppError> {
    let row = svc()
        .update_department(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Department updated"))
}

async fn update_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateJobTitle>,
) -> Result<Response, AppError> {
    let row = svc()
        .update_job_title(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Job title updated"))
}

async fn delete_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    svc()
        .delete_department(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Department deleted"))
}

async fn delete_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    svc()
        .delete_job_title(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Job title deleted"))
}

// =============================================================================
// EMPLOYEES
// =============================================================================
async fn create_employee(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_employee(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Employee created"))
}

async fn list_employees(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_employees(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(rows, "Employees fetched"))
}

async fn get_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .get_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(row, "Employee fetched"))
}

async fn update_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateEmployee>,
) -> Result<Response, AppError> {
    let row = svc()
        .update_employee(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Employee updated"))
}

async fn delete_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    svc()
        .delete_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}

// =============================================================================
// PROFILES
// =============================================================================
async fn upsert_statutory_profile(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpsertStatutoryProfile>,
) -> Result<Response, AppError> {
    let row = svc()
        .upsert_statutory_profile(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Statutory profile saved"))
}

async fn get_statutory_profile(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .get_statutory_profile(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(row, "Statutory profile fetched"))
}

async fn upsert_payment_profile(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpsertPaymentProfile>,
) -> Result<Response, AppError> {
    let row = svc()
        .upsert_payment_profile(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Payment profile saved"))
}

async fn get_payment_profile(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .get_payment_profile(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(row, "Payment profile fetched"))
}

async fn get_lifecycle(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc().get_lifecycle(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(row, "Lifecycle fetched"))
}

async fn list_employment_events(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_employment_events(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(rows, "Employment events fetched"))
}

// =============================================================================
// CONTRACTS
// =============================================================================
async fn create_contract(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEmploymentContract>,
) -> Result<Response, AppError> {
    let row = svc().create_contract(&user.tenant_pool, &payload).await?;
    Ok(ApiResponse::created(row, "Contract created"))
}

async fn list_contracts(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<EmployeeFilter>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_contracts(&user.tenant_pool, q.employee_uuid)
        .await?;
    Ok(ApiResponse::success(rows, "Contracts fetched"))
}

async fn get_contract(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc().get_contract(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(row, "Contract fetched"))
}

async fn add_amendment(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateContractAmendment>,
) -> Result<Response, AppError> {
    let row = svc()
        .add_amendment(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Amendment recorded"))
}

// =============================================================================
// LEAVE
// =============================================================================
async fn create_leave_type(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateLeaveType>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_leave_type(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Leave type created"))
}

async fn list_leave_types(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc().list_leave_types(&user.tenant_pool).await?;
    Ok(ApiResponse::success(rows, "Leave types fetched"))
}

async fn upsert_leave_balance(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpsertLeaveBalance>,
) -> Result<Response, AppError> {
    let row = svc()
        .upsert_leave_balance(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Leave balance saved"))
}

async fn list_leave_balances(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<EmployeeFilter>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_leave_balances(&user.tenant_pool, q.employee_uuid, q.year)
        .await?;
    Ok(ApiResponse::success(rows, "Leave balances fetched"))
}

async fn create_leave_request(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateLeaveRequest>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_leave_request(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Leave request created"))
}

async fn list_leave_requests(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<EmployeeFilter>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_leave_requests(&user.tenant_pool, q.employee_uuid)
        .await?;
    Ok(ApiResponse::success(rows, "Leave requests fetched"))
}

async fn decide_leave_request(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<DecideLeaveRequest>,
) -> Result<Response, AppError> {
    let row = svc()
        .decide_leave_request(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Leave request decided"))
}

async fn create_leave_schedule(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateLeaveSchedule>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_leave_schedule(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Leave schedule created"))
}

async fn list_leave_schedules(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<EmployeeFilter>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_leave_schedules(&user.tenant_pool, q.employee_uuid, q.year)
        .await?;
    Ok(ApiResponse::success(rows, "Leave schedules fetched"))
}

async fn update_leave_schedule_status(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateLeaveScheduleStatus>,
) -> Result<Response, AppError> {
    let row = svc()
        .update_leave_schedule_status(&user.tenant_pool, id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Leave schedule updated"))
}

async fn create_leave_encashment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateLeaveEncashment>,
) -> Result<Response, AppError> {
    let row = svc()
        .create_leave_encashment(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Leave encashment created"))
}

async fn list_leave_encashments(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<EmployeeFilter>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_leave_encashments(
            &user.tenant_pool,
            q.employee_uuid,
            q.status.as_deref(),
        )
        .await?;
    Ok(ApiResponse::success(rows, "Leave encashments fetched"))
}

// =============================================================================
// LIFECYCLE
// =============================================================================
async fn list_onboarding_tasks(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_onboarding_tasks(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(rows, "Onboarding tasks fetched"))
}

async fn complete_onboarding_task(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CompleteChecklistTask>,
) -> Result<Response, AppError> {
    let row = svc()
        .complete_onboarding_task(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Onboarding task completed"))
}

async fn complete_onboarding(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .complete_onboarding(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(row, "Employee activated"))
}

async fn start_exit(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<StartExit>,
) -> Result<Response, AppError> {
    let row = svc()
        .start_exit(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Exit started"))
}

async fn list_offboarding_tasks(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = svc()
        .list_offboarding_tasks(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(rows, "Offboarding tasks fetched"))
}

async fn complete_offboarding_task(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CompleteChecklistTask>,
) -> Result<Response, AppError> {
    let row = svc()
        .complete_offboarding_task(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Offboarding task completed"))
}

async fn complete_exit(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let row = svc()
        .complete_exit(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(row, "Exit completed"))
}








// src/features/hr/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::workforce::{repository::PostgresHrRepo, services::HrService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::employee::{CreateDepartment, CreateEmployee, CreateJobTitle},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/employee", post(create_employee))
        .route("/create/department", post(create_department))
        .route("/create/jobtitle", post(create_job_title))
        .route("/list/employees", get(list_employees))
        .route("/list/departments", get(list_departments))
        .route("/list/jobtitles", get(list_job_titles))
        .route("/get/employee/{id}", get(get_employee))
        .route("/get/department/{id}", get(get_department))
        .route("/get/jobtitle/{id}", get(get_job_title))
        .route("/update/employee/{id}", patch(update_employee))
        .route("/update/department/{id}", patch(update_department))
        .route("/update/jobtitle/{id}", patch(update_job_title))
        .route("/delete/employee/{id}", delete(delete_employee))
        .route("/delete/department/{id}", delete(delete_department))
        .route("/delete/jobtitle/{id}", delete(delete_job_title))
}

async fn create_employee(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .create_employee(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(emp, "Employee created"))
}

async fn create_department(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDepartment>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let department = svc
        .create_department(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(department, "Department created"))
}

async fn create_job_title(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateJobTitle>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let job_title = svc
        .create_job_title(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(job_title, "Job title created"))
}

async fn list_employees(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let list = svc.list_employees(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Employees fetched"))
}

async fn list_departments(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let list = svc
        .list_departments(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Employees fetched"))
}
async fn list_job_titles(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let list = svc.list_job_titles(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Employees fetched"))
}

async fn get_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .get_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(emp, "Employee fetched"))
}

async fn get_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .get_department(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(emp, "Employee fetched"))
}
async fn get_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc.get_job_title(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(emp, "Employee fetched"))
}

async fn update_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .update_employee(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(emp, "Employee updated"))
}

async fn update_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDepartment>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .update_department(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(emp, "Employee updated"))
}

async fn update_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateJobTitle>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .update_job_title(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(emp, "Employee updated"))
}

async fn delete_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    svc.delete_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}

async fn delete_department(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    svc.delete_department(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}

async fn delete_job_title(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    svc.delete_job_title(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}
