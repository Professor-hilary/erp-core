// src/features/payroll/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, Query, State},
    response::Response,
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::payroll::{repository::PostgresPayrollRepo, services::PayrollService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::payroll::*,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Payrun lifecycle
        .route("/payrun/create", post(create_payrun))
        .route("/payrun/process/{serial_id}", post(process_payrun))
        .route("/payrun/post", post(post_payrun))
        .route("/get/payrun/{uuid}", get(get_payrun))
        .route("/list/payruns", get(list_payruns))
        .route("/list/payslips", get(list_payslips))
        .route("/payrun/{uuid}/payslips", get(list_payslips_for_payrun))
        // Components
        .route(
            "/components",
            post(create_pay_component).get(list_pay_components),
        )
        .route(
            "/assignments",
            post(create_component_assignment).get(list_component_assignments),
        )
        // Payslip breakdown
        .route(
            "/payslip/{uuid}/calculation-items",
            get(get_payslip_calculation_items),
        )
        // Settlement
        .route("/settle", post(settle_employees))
        .route("/payrun/{uuid}/payment-batch", get(get_payment_batch))
        // Statutory remittance
        .route("/remittances/create", post(create_statutory_remittances))
        .route("/remittances/pay", post(remit_statutory))
        .route("/remittances", get(list_statutory_remittances))
}

// ---------------------------------------------------------------------------
// Payrun
// ---------------------------------------------------------------------------

async fn create_payrun(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreatePayrun>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
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
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let payrun = service.process_payrun(&user.tenant_pool, payrun_id).await?;
    Ok(ApiResponse::success(payrun, "Payrun processed"))
}

async fn post_payrun(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostPayrun>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    service
        .post_payrun(user.user_id, &user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success((), "Payrun posted to GL"))
}

async fn get_payrun(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let payrun = service.get_payrun(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(payrun, "Payrun fetched"))
}

async fn list_payruns(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let payruns = service.get_payruns(&user.tenant_pool).await?;
    Ok(ApiResponse::success(payruns, "Payruns fetched"))
}

async fn list_payslips(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let slips = service.get_payslips(&user.tenant_pool).await?;
    Ok(ApiResponse::success(slips, "Payslips fetched"))
}

async fn list_payslips_for_payrun(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let slips = service
        .get_payslips_for_payrun(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(slips, "Payslips fetched"))
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

async fn create_pay_component(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreatePayComponent>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let component = service
        .create_pay_component(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(component, "Pay component created"))
}

async fn list_pay_components(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let components = service.list_pay_components(&user.tenant_pool).await?;
    Ok(ApiResponse::success(components, "Pay components fetched"))
}

async fn create_component_assignment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateComponentAssignment>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let assignment = service
        .create_component_assignment(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(assignment, "Component assignment created"))
}

#[derive(Debug, Deserialize)]
struct AssignmentQuery {
    employee_uuid: Option<Uuid>,
}

async fn list_component_assignments(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<AssignmentQuery>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let rows = service
        .list_component_assignments(&user.tenant_pool, q.employee_uuid)
        .await?;
    Ok(ApiResponse::success(rows, "Component assignments fetched"))
}

// ---------------------------------------------------------------------------
// Payslip breakdown
// ---------------------------------------------------------------------------

async fn get_payslip_calculation_items(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let items = service
        .get_payslip_calculation_items(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(items, "Calculation items fetched"))
}

// ---------------------------------------------------------------------------
// Settlement
// ---------------------------------------------------------------------------

async fn settle_employees(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<SettleEmployees>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let batch = service
        .settle_employees(user.user_id, &user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(batch, "Employees settled"))
}

async fn get_payment_batch(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let batch = service.get_payment_batch(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(batch, "Payment batch fetched"))
}

// ---------------------------------------------------------------------------
// Statutory remittance
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CreateRemittancesBody {
    payrun_uuid: Uuid,
    due_date: NaiveDate,
}

async fn create_statutory_remittances(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateRemittancesBody>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let rows = service
        .create_statutory_remittances(&user.tenant_pool, payload.payrun_uuid, payload.due_date)
        .await?;
    Ok(ApiResponse::created(rows, "Statutory remittances created"))
}

async fn remit_statutory(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<RemitStatutory>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let row = service
        .remit_statutory(user.user_id, &user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Statutory liability remitted"))
}

#[derive(Debug, Deserialize)]
struct RemittanceQuery {
    payrun_uuid: Option<Uuid>,
}

async fn list_statutory_remittances(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<RemittanceQuery>,
) -> Result<Response, AppError> {
    let service = PayrollService::new(PostgresPayrollRepo::new());
    let rows = service
        .list_statutory_remittances(&user.tenant_pool, q.payrun_uuid)
        .await?;
    Ok(ApiResponse::success(rows, "Statutory remittances fetched"))
}
