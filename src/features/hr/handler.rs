// src/features/hr/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;

use crate::{
    features::hr::{repository::PostgresHrRepo, service::HrService},
    infrastructure::errors::AppError,
    infrastructure::responses::ApiResponse,
    middleware::auth::AuthenticatedTenant,
    models::{employee::CreateEmployee, payrun::CreatePayrun},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_employee))
        .route("/list", get(list_employees))
        .route("/get/{id}", get(get_employee))
        .route("/update/{id}", patch(update_employee))
        .route("/delete/{id}", delete(delete_employee))
        .route("/payruns", post(create_and_post_payrun))
        .route("/payruns/{id}", get(get_payrun))
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

async fn list_employees(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let list = svc.list_employees(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Employees fetched"))
}

async fn get_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let emp = svc
        .get_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(emp, "Employee fetched"))
}

async fn update_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
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

async fn delete_employee(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    svc.delete_employee(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}

async fn create_and_post_payrun(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreatePayrun>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let payrun = svc
        .create_and_post_payrun(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(
        payrun,
        "Payrun created and posted to GL",
    ))
}

async fn get_payrun(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new();
    let svc = HrService::new(repo);
    let payrun = svc.get_payrun(&user.tenant_pool, id, user.user_id).await?;
    Ok(ApiResponse::success(payrun, "Payrun fetched"))
}
