// src/features/accounts/handlers.rs
use crate::{
    features::manufacturing::{repository::ManufacturingRepo, services::ManufacturingService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::manufacturing::{
        ApplyOverheadDto, CompleteProductionOrderDto, CreateProductionOrderDto, IssueMaterialDto,
        ProrateVarianceDto,
    },
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Json, State},
    response::Response,
    routing::{/* delete, get, patch, */ post},
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/production-order", post(new_order_route))
        .route("/variance-allocation", post(prorate_variance_route))
        .route("/materials-issuance", post(raw_material_issuance_route))
        .route("/overhead-application", post(apply_overhead_route))
        .route("/complete-production", post(complete_order_route))
}

async fn new_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateProductionOrderDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.create_order(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Order Created")),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

async fn prorate_variance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<ProrateVarianceDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.prorate_variance(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Proration Successful")),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

async fn raw_material_issuance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<IssueMaterialDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.issue_material(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Material Issuance Successful")),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

async fn apply_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<ApplyOverheadDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.apply_overhead(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Overhead Applied")),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}

async fn complete_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CompleteProductionOrderDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.complete_order(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Completed")),
        Err(e) => Err(AppError::Internal(e.to_string())),
    }
}
