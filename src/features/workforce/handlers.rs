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
    let emp = svc
        .get_job_title(&user.tenant_pool, id)
        .await?;
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
