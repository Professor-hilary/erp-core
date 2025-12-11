use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::reports::{repository::PostgresReportRepo, service::ReportService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/balancesheet", post(get_balancesheet))
        .route("/income", get(get_income))
        .route("/cashflow/direct", get(get_cf_direct))
        .route("/cashflow/indirect", get(get_cf_indirect))
        .route("/aging/receivables", patch(get_aging_ar))
        .route("/aging/payables", patch(get_aging_ap))
        .route("/sales", delete(get_sales_report))
        .route("/purchase", delete(get_purchases_report))
}

async fn get_balancesheet(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .balancesheet(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::created(report, "Report created"))
}

async fn get_income(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let reports = service
        .income_statement(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(reports, "Reports fetched"))
}

async fn get_aging_ap(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .aging_ap(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn get_aging_ar(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .aging_ar(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn get_cf_direct(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .cf_direct(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report updated"))
}

async fn get_cf_indirect(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    service
        .cf_indirect(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Report deleted"))
}

async fn get_sales_report(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    service
        .sales_report(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Report deleted"))
}

async fn get_purchases_report(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    service
        .purchase_report(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Report deleted"))
}
