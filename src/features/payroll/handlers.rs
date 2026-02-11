// src/features/payroll/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::payroll::{repository::PostgresPayrollRepo, services::PayrollService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::payrun::{CreatePayrun, Payrun, Payslip, PostPayrun},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/payrun/create", post(create_payrun))
        .route("/payrun/process/{uuid}", post(process_payrun))
        .route("/payrun/post", post(post_payrun))
        .route("/get/payrun/{uuid}", get(get_payrun))
        .route("/list/payruns", get(list_payruns))
        .route("/list/payslips", get(list_payslips))
}

async fn create_payrun(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreatePayrun>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun = service
        .create_payrun(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(payrun, "Payrun created"))
}

async fn process_payrun(
    State(_state): State<Arc<AppState>>,
    Path(payrun_id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun = service.process_payrun(&user.tenant_pool, payrun_id).await?;
    Ok(ApiResponse::success(payrun, "Payrun processed"))
}

async fn get_payrun(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun: Payrun = service
        .get_payrun(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(payrun, "Payrun fetched"))
}

async fn list_payruns(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun: Vec<Payrun> = service.get_payruns(&user.tenant_pool).await?;
    Ok(ApiResponse::success(payrun, "Payrun list fetched"))
}

async fn list_payslips(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun: Vec<Payslip> = service.get_payslips(&user.tenant_pool).await?;
    Ok(ApiResponse::success(payrun, "Payrun list fetched"))
}

async fn post_payrun(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostPayrun>,
) -> Result<Response, AppError> {
    let repo = PostgresPayrollRepo::new();
    let service = PayrollService::new(repo);
    let payrun: () = service
        .post_payrun(user.user_id, &user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(payrun, "Payrun posted to GL"))
}
