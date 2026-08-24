use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{get, post},
};

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::capital::{repository::PostgresCapitalRepo, services::CapitalService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::capital::{
        CapitalFacility, CreateCapitalFacility, CreateDrawdown, CreateRepayment, DebtDrawdown,
        DebtRepayment,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Facilities
        .route("/facility/create", post(create_facility))
        .route("/facility/{uuid}", get(get_facility))
        .route("/list/facilities", get(list_facilities))
        // Drawdowns
        .route("/drawdown/create", post(create_drawdown))
        .route("/list/drawdowns/{uuid}", get(list_drawdowns))
        // Repayments
        .route("/repayment/create", post(create_repayment))
        .route("/list/repayments/{uuid}", get(list_repayments))
}

// ---------------------------------------------------------
// Facilities
// ---------------------------------------------------------

async fn create_facility(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalFacility>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let facility = service
        .create_facility(&user.tenant_pool, user.user_id, &payload)
        .await?;

    Ok(ApiResponse::created(facility, "Capital facility created"))
}

async fn get_facility(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let facility: CapitalFacility = service.get_facility(&user.tenant_pool, id).await?;

    Ok(ApiResponse::success(facility, "Capital facility fetched"))
}

async fn list_facilities(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let facilities: Vec<CapitalFacility> = service.get_facilities(&user.tenant_pool).await?;

    Ok(ApiResponse::success(
        facilities,
        "Capital facilities fetched",
    ))
}

// ---------------------------------------------------------
// Drawdowns
// ---------------------------------------------------------

async fn create_drawdown(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDrawdown>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let drawdown = service
        .create_drawdown(&user.tenant_pool, user.user_id, &payload)
        .await?;

    Ok(ApiResponse::created(drawdown, "Debt drawdown created"))
}

async fn list_drawdowns(
    State(_state): State<Arc<AppState>>,
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let drawdowns: Vec<DebtDrawdown> = service
        .get_drawdowns(&user.tenant_pool, facility_id)
        .await?;

    Ok(ApiResponse::success(drawdowns, "Debt drawdowns fetched"))
}

// ---------------------------------------------------------
// Repayments
// ---------------------------------------------------------

async fn create_repayment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateRepayment>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let repayment = service
        .create_repayment(&user.tenant_pool, user.user_id, &payload)
        .await?;

    Ok(ApiResponse::created(repayment, "Debt repayment created"))
}

async fn list_repayments(
    State(_state): State<Arc<AppState>>,
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCapitalRepo::new();
    let service = CapitalService::new(repo);

    let repayments: Vec<DebtRepayment> = service
        .get_repayments(&user.tenant_pool, facility_id)
        .await?;

    Ok(ApiResponse::success(repayments, "Debt repayments fetched"))
}
