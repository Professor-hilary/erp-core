// src/features/reports/handlers.rs
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::reports::{
        repository::{ForceReload, PostgresReportRepo},
        services::ReportService,
    },
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AsOfQuery {
    pub as_of: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/materialize-reports", post(refresh_reports_manual))
        .route("/balancesheet/single", get(http_balancesheet))
        .route("/balancesheet/compare", get(http_balancesheet_compare))
        .route("/income", get(http_income))
        .route("/cashflow/direct", get(http_cf_direct))
        .route("/cashflow/indirect", get(http_cf_indirect))
        .route("/aging/receivables", get(http_aging_ar))
        .route("/aging/payables", get(http_aging_ap))
        .route("/sales", get(http_sales_report))
        .route("/purchase", get(http_purchases_report))
        .route("/trial-balance", get(http_trial_balance))
        .route("/inventory-valuation", get(http_inventory_valuation))
        .route("/customer-statement", get(http_customer_report))
        .route("/payroll", get(http_payroll))
        .route("/change-of-equity", get(http_change_equity))
}

fn parse_date_opt(s: Option<String>) -> Option<NaiveDate> {
    s.and_then(|x| NaiveDate::parse_from_str(&x, "%Y-%m-%d").ok())
}

/// GET /api/reports/balancesheet/single?as_of=yyyy-mm-dd
async fn http_balancesheet(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<AsOfQuery>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let as_of = q
        .as_of
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let as_of = NaiveDate::parse_from_str(&as_of, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let report = service
        .balancesheet(&user.tenant_pool, as_of)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::created(report, "Report created"))
}

/// GET /api/reports/balancesheet?start=yyyy-mm-dd&end=yyyy-mm-dd
async fn http_balancesheet_compare(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let start = q
        .from
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());
    let end =
        q.to.unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let start = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;
    let end = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let report = service
        .balancesheet_compare(&user.tenant_pool, start, end)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(report, "Report updated"))
}

/// GET /api/reports/income?start=yyyy-mm-dd&end=yyyy-mm-dd
async fn http_income(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let start = q
        .from
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());
    let end =
        q.to.unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let start = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;
    let end = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let report = service
        .income_statement(&user.tenant_pool, start, end)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(report, "Reports fetched"))
}

async fn http_aging_ap(
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

async fn http_aging_ar(
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

async fn http_cf_direct(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let start = q
        .from
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());
    let end =
        q.to.unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let start = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;
    let end = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let report = service
        .cf_direct(&user.tenant_pool, start, end)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(report, "Report updated"))
}

async fn http_cf_indirect(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let start = q
        .from
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());
    let end =
        q.to.unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let start = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;
    let end = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let report = service
        .cf_indirect(&user.tenant_pool, start, end)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(
        report,
        "Indirect CashFlow Statement fetched",
    ))
}

async fn http_change_equity(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<DateRangeQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let start = q
        .from
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());
    let end =
        q.to.unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let start = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;
    let end = NaiveDate::parse_from_str(&end, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let report = service
        .change_of_equity(&user.tenant_pool, start, end)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(
        report,
        "Change of Equity Statement fetched",
    ))
}

async fn http_sales_report(
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

async fn http_purchases_report(
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
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<ForceReload>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);
    service.refresh_all(&user.tenant_pool, &payload).await?;

    Ok(ApiResponse::success((), "Reports refreshed"))
}

/// GET /api/reports/trial-balance?as_of=yyyy-mm-dd
async fn http_trial_balance(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<AsOfQuery>,
) -> Result<Response, AppError> {
    let repo: PostgresReportRepo = PostgresReportRepo::new();
    let service: ReportService<PostgresReportRepo> = ReportService::new(repo);

    let as_of = q
        .as_of
        .unwrap_or_else(|| chrono::Local::now().naive_local().to_string());

    let as_of = NaiveDate::parse_from_str(&as_of, "%Y-%m-%d")
        .map_err(|e| AppError::BadRequest(format!("invalid date: {}", e)))?;

    let report = service
        .trial_balance(&user.tenant_pool, as_of)
        .await
        .map_err(|e| AppError::Internal(format!("{}", e.to_string())))?;

    Ok(ApiResponse::success(report, "Report fetched"))
}

async fn http_inventory_valuation(
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

async fn http_customer_report(
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

async fn http_payroll(
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
