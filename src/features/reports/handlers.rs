// src/features/reports/handlers.rs
use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::reports::{repository::PostgresReportRepo, services::ReportService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

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
        .route("/refresh-reports", delete(refresh_reports_manual))
        .route("/trial-balance", delete(get_trial_balance))
        .route("/inventory-valuation", delete(get_inventory_valuation))
        .route("/customer-statement", delete(get_customer_report))
        .route("/payroll", delete(get_payroll))
}

fn parse_date_opt(s: Option<String>) -> Option<NaiveDate> {
    s.and_then(|x| NaiveDate::parse_from_str(&x, "%Y-%m-%d").ok())
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
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let from = parse_date_opt(q.from);
    let to = parse_date_opt(q.to);

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let reports = service
        .income_statement(&user.tenant_pool, user.user_id, from, to)
        .await?;
    Ok(ApiResponse::success(reports, "Reports fetched"))
}

async fn get_aging_ap(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(uuid): Path<Uuid>,
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
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let from = parse_date_opt(q.from);
    let to = parse_date_opt(q.to);

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .cf_direct(&user.tenant_pool, uuid, user.user_id, from, to)
        .await?;
    Ok(ApiResponse::success(report, "Report updated"))
}

async fn get_cf_indirect(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let from = parse_date_opt(q.from);
    let to = parse_date_opt(q.to);

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .cf_indirect(&user.tenant_pool, uuid, user.user_id, from, to)
        .await?;
    Ok(ApiResponse::success(report, "Report deleted"))
}

async fn get_sales_report(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let from = parse_date_opt(q.from);
    let to = parse_date_opt(q.to);

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .sales_report(&user.tenant_pool, uuid, user.user_id, from, to)
        .await?;
    Ok(ApiResponse::success(report, "Report deleted"))
}

async fn get_purchases_report(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let from = parse_date_opt(q.from);
    let to = parse_date_opt(q.to);

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .purchase_report(&user.tenant_pool, uuid, user.user_id, from, to)
        .await?;
    Ok(ApiResponse::success(report, "Report deleted"))
}

async fn refresh_reports_manual(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>, // require admin
) -> Result<Response, AppError> {
    // verify user is admin tenant owner; omitted here
    let pool = &user.tenant_pool;

    // run refreshes (copied from worker)
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY reporting.balance_sheet")
        .execute(pool)
        .await?;
    // ... repeat for rest
    Ok(ApiResponse::success((), "Reports refreshed"))
}

async fn get_trial_balance(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(_uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .trial_balance(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn get_inventory_valuation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(_uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .inventory_valuation(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn get_customer_report(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(_uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .customer_statement(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn get_payroll(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(_uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    let report = service
        .payroll_statement(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(report, "Report fetched"))
}
