// src/features/accounts/handlers.rs
use crate::{
    features::manufacturing::{repository::ManufacturingRepo, services::ManufacturingService},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::manufacturing::{
        ApplyLaborCostDto, ApplyOverheadDto, CompleteProductionOrderDto, CreateBomHeaderDto,
        CreateBomLineDto, CreateOverheadRateDto, CreateProductionOrderDto, IssueMaterialDto,
        ProrateVarianceDto, RecognizeOverhead,
    },
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{/* delete, patch, */ get, post},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct CreateBomPayload {
    pub header: CreateBomHeaderDto,
    pub lines: Vec<CreateBomLineDto>,
}

// Helper for displaying clean json error messages
fn internal_error(e: impl std::fmt::Display) -> AppError {
    tracing::error!("Internal error: {}", e);
    AppError::Internal(e.to_string())
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/production-order", post(new_order_route))
        .route("/create/overhead-rate", post(new_overhead_rate_route))
        .route("/variance-allocation", post(prorate_variance_route))
        .route("/materials-issuance", post(raw_material_issuance_route))
        .route("/apply/control-wip", post(apply_overhead_route))
        .route("/apply/overhead-control", post(recognize_overhead_route))
        .route("/apply/labor-cost", post(apply_labor_route))
        .route("/complete-production", post(complete_order_route))
        .route("/create/bom", post(create_bom_route))
        .route("/bom/{uuid}", get(get_bom_route))
        .route("/default-product/{uuid}", get(default_bom_route))
}

///# POST /manufacturing/create/bom
/// 1st. STEP: This is the first step, create Bill Oof Materials for production  from Raw Materials available
/// manufacturing.boms and manufacturing.bom_lines are updated. Example: `PRODUCT = CHAIR`
/// > 1. Wood plank -> 5 units
/// > 2. Nails -> 20 units
/// > 3. Vanish -> 1 unit
/// > 4. Cloth -> 1 unit
async fn create_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateBomPayload>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service
        .create_bom(payload.header, payload.lines, user.user_id)
        .await
    {
        Ok(result) => Ok(ApiResponse::created(result, "BOM Created")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/create/production-order
/// 2nd. STEP: Create a production order, define order no, product id, qty, start and expectdd end date
/// status (Use [Plammed], Completed, in Process, Cancelled), general ledger account
async fn new_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateProductionOrderDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.create_order(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Order Created")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/materials-issuance
/// 3rd Prepare Materials for production, Debit WIP Credit Raw Materials
async fn raw_material_issuance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<IssueMaterialDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.issue_material(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Material Issuance Successful")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/create/overhead-rate
async fn new_overhead_rate_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateOverheadRateDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.create_overhead_rate(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Order Created")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/variance-allocation
async fn prorate_variance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ProrateVarianceDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.prorate_variance(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Proration Successful")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/overhead-application
async fn recognize_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RecognizeOverhead>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.recognize_expenditures(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(
            result,
            "Indirect Cost Applied to Control Account",
        )),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/overhead-application
async fn apply_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyOverheadDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.apply_overhead(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Overhead Applied")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/overhead-application
async fn apply_labor_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyLaborCostDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.apply_labor_costs(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Labor Costs Applied")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/complete-production
async fn complete_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CompleteProductionOrderDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.complete_order(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Completed")),
        Err(e) => Err(internal_error(e)),
    }
}

/// GET /manufacturing/bom/:uuid
async fn get_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.get_bom(uuid).await {
        Ok(Some(result)) => Ok(ApiResponse::success(result, "BOM Created")),
        Ok(None) => Err(AppError::NotFound("BOM not found".into())),
        Err(e) => Err(internal_error(e)),
    }
}

///# GET /manufacturing/default-product/{uuid}
async fn default_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.default_bom(uuid).await {
        Ok(Some(result)) => Ok(ApiResponse::success(result, "BOM found")),
        Ok(None) => Err(AppError::NotFound("No default OM found".into())),
        Err(e) => Err(internal_error(e)),
    }
}
