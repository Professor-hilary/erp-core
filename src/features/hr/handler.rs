// src/features/hr/handlers.rs
use axum::{
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use std::sync::Arc;

use crate::{
    errors::AppError,
    features::hr::{repository::PostgresHrRepo, service::HrService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::{employee::{CreateEmployee}, payrun::{CreatePayrun}},
    routes::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/employees", post(create_employee))
        .route("/employees", get(list_employees))
        .route("/employees/:id", get(get_employee))
        .route("/employees/:id", patch(update_employee))
        .route("/employees/:id", delete(delete_employee))
        .route("/payruns", post(create_and_post_payrun))
        .route("/payruns/:id", get(get_payrun))
}

async fn create_employee(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let emp = svc.create_employee(user.0, &payload).await?;
    Ok(ApiResponse::created(emp, "Employee created"))
}

async fn list_employees(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let list = svc.list_employees(user.0).await?;
    Ok(ApiResponse::success(list, "Employees fetched"))
}

async fn get_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let emp = svc.get_employee(id, user.0).await?;
    Ok(ApiResponse::success(emp, "Employee fetched"))
}

async fn update_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateEmployee>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let emp = svc.update_employee(id, user.0, &payload).await?;
    Ok(ApiResponse::success(emp, "Employee updated"))
}

async fn delete_employee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    svc.delete_employee(id, user.0).await?;
    Ok(ApiResponse::success((), "Employee deleted"))
}

async fn create_and_post_payrun(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreatePayrun>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let payrun = svc.create_and_post_payrun(user.0, &payload, &state.pool).await?;
    Ok(ApiResponse::created(payrun, "Payrun created and posted to GL"))
}

async fn get_payrun(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresHrRepo::new(state.pool.clone());
    let svc = HrService::new(repo);
    let payrun = svc.get_payrun(id, user.0).await?;
    Ok(ApiResponse::success(payrun, "Payrun fetched"))
}