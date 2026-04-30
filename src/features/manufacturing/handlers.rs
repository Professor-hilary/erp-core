// src/features/accounts/handlers.rs
use crate::{
    features::manufacturing::{repository::ManufacturingRepo, services::ManufacturingService},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::manufacturing::*,
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
        .route("/create/routing/workcenter", post(new_work_center))
        .route("/create/routing/header", post(new_routing))
        .route("/create/routing/operation", post(new_routing_operation))
        .route("/create/overhead-rate", post(new_overhead_rate_route))
        .route("/variance-allocation", post(prorate_variance_route))
        .route(
            "/apply/direct/material",
            post(bulk_raw_material_issuance_route),
        )
        .route(
            "/apply/direct/material/single",
            post(raw_material_issuance_route),
        )
        .route("/apply/direct/labor", post(http_direct_labor_route))
        // .route("/apply/indirect/labor", post(http_indirect_labor_route))
        .route("/apply/overhead/actual", post(actual_overhead_route))
        .route("/apply/overhead/applied", post(applied_overhead_route))
        .route("/complete-production", post(complete_order_route))
        .route("/create/bom", post(create_bom_route))
        .route("/get/bom/{uuid}", get(get_bom_route))
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

///# POST /manufacturing/create/routing/workcenter
async fn new_work_center(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<WorkCenterDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_work_center(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Work Center Created")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/create/routing/header
async fn new_routing(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RoutingsDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_routing(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Routing Header Created")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/create/routing/operation
async fn new_routing_operation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RoutingOperationsDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_routing_operation(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Routing Operation Created")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/materials-issuance
/// 3rd Prepare Materials for production, Debit WIP Credit Raw Materials
async fn bulk_raw_material_issuance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<IssueMaterialDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.bulk_issue_material(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Material Issuance Successful")),
        Err(e) => Err(internal_error(e)),
    }
}

///# POST /manufacturing/materials-issuance
/// 3rd Prepare Materials for production, Debit WIP Credit Raw Materials
async fn raw_material_issuance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<SingleMaterialIssueDto>,
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
/// This is invoiced and paid overhead by the accountant or resource disbursement
/// which may differ from rated overhead, typically ran when money is available
async fn actual_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RecognizeOverhead>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.recognize_overhead_paid(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Actual Overhead Recognized")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/overhead-application
/// Used with overhead rates to cost production, such as, based on machine hour,
/// labor, material cost, etc. Assumed Overhead is approximated rather than actual.
async fn applied_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyOverheadDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.apply_rated_overhead(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Applied Overhead Applied")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/apply/direct/labor
async fn http_direct_labor_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyDirectLaborDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service
        .apply_direct_labor_costs(payload, user.user_id)
        .await
    {
        Ok(result) => Ok(ApiResponse::created(result, "Direct Labor Costs Applied")),
        Err(e) => Err(internal_error(e)),
    }
}

/// POST /manufacturing/apply/indirect/labor
// async fn http_indirect_labor_route(
//     State(_state): State<Arc<AppState>>,
//     Extension(user): Extension<AuthenticatedTenant>,
//     AppJson(payload): AppJson<ApplyInDirectLaborDto>,
// ) -> Result<Response, AppError> {
//     let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
//     let service: ManufacturingService = ManufacturingService::new(repo);

//     match service.apply_indirect_labor_costs(payload, user.user_id).await {
//         Ok(result) => Ok(ApiResponse::created(result, "Inirect Labor Costs Applied")),
//         Err(e) => Err(internal_error(e)),
//     }
// }

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
