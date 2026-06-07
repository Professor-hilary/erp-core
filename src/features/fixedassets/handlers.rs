// src/routes/vendors/handler.rs
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::fixedassets::{
        repository::PostgresFixedAssetRepository, services::FixedAssetService,
    },
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::fixed_assets::*,
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/asset/create", post(http_create_asset))
        .route("/asset/update", patch(http_update_asset))
        .route("/asset/list", get(http_list_assets))
        .route("/asset/get/{uuid}", get(http_get_asset))
        .route("/asset/delete", delete(http_delete_asset))
        .route("/asset/class/create", post(http_create_asset_class))
        .route("/asset/class/update", patch(http_update_asset_class))
        .route("/asset/class/list", get(http_list_asset_classes))
        .route("/asset/class/get/{uuid}", get(http_get_asset_class))
        .route("/asset/class/delete", delete(http_delete_asset_class))
        .route("/asset/book/create", post(http_create_asset_book))
        .route("/asset/book/update", patch(http_update_asset_book))
        .route("/asset/book/list", get(http_list_asset_books))
        .route("/asset/book/get/{uuid}", get(http_get_asset_book))
        .route("/asset/book/delete", delete(http_delete_asset_book))
        .route("/cwip/create", post(http_create_asset_cwip))
        .route("/cwip/update", patch(http_update_asset_cwip))
        .route("/cwip/list", get(http_list_asset_cwip))
        .route("/cwip/get/{uuid}", get(http_get_asset_cwip))
        .route("/cwip/capitalize/{uuid}", post(http_capitalize_cwip))
        .route("/cwip/delete", delete(http_delete_asset_cwip))
        .route("/depreciation/compute", post(http_compute_depreciation))
        .route("/depreciation/run", post(http_run_depreciation))
        .route("/depreciation/schedule/{uuid}", get(http_get_depr_schedule))
        .route("/depreciation/post/{uuid}", post(http_post_depr_to_gl))
        .route("/component/create", post(http_create_asset_component))
        .route("/component/update", patch(http_update_asset_component))
        .route("/component/list", get(http_list_asset_component))
        .route("/component/get/{uuid}", get(http_get_asset_component))
        .route("/component/delete", delete(http_delete_asset_component))
        .route("/insurance/create", post(http_create_insurance))
        .route("/insurance/update", patch(http_update_insurance))
        .route("/insurance/list", get(http_list_insurance))
        .route("/insurance/get/{uuid}", get(http_get_insurance))
        .route("/insurance/delete", delete(http_delete_insurance))
        .route("/maintenance/create", post(http_create_maintenance))
        .route("/maintenance/update", patch(http_update_maintenance))
        .route("/maintenance/list", get(http_list_maintenance))
        .route("/maintenance/get/{uuid}", get(http_get_maintenance))
        .route("/maintenance/delete", delete(http_delete_maintenance))
        .route("/transfer/create", post(http_transfer_asset))
        .route("/transfer/history", get(http_transfer_history))
        .route("/transaction/history", get(http_transaction_history))
        .route("/revaluation/create", post(http_create_revaluation))
        .route("/revaluation/history", get(http_revaluation_history))
        .route("/disposal/create", post(http_dispose_asset))
        .route("/disposal/history", get(http_disposal_history))
}

async fn http_create_asset(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAsset>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .new_asset(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Asset created"))
}

async fn http_list_assets(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service.list_asset(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Assets fetched"))
}

async fn http_get_asset(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_asset(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Asset fetched"))
}

async fn http_update_asset(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAsset>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_asset(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Asset updated"))
}

async fn http_delete_asset(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_asset(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Asset Deleted", "Asset deleted"))
}

async fn http_create_asset_class(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetClass>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .new_asset_class(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Asset class created"))
}

async fn http_list_asset_classes(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_asset_classes(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Assets class fetched"))
}

async fn http_get_asset_class(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_asset_class(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Asset class fetched"))
}

async fn http_update_asset_class(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetClass>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_asset_class(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Asset class updated"))
}

async fn http_delete_asset_class(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_asset_class(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Asset Deleted", "Asset class deleted"))
}

async fn http_create_asset_book(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetBook>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .new_asset_book(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Asset book created"))
}

async fn http_list_asset_books(
    State(_state): State<Arc<AppState>>,
    Path(asset_uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_asset_books(&user.tenant_pool, asset_uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Assets book fetched"))
}

async fn http_get_asset_book(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_asset_book(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Asset book fetched"))
}

async fn http_update_asset_book(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetBook>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_asset_book(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Vendor book updated"))
}

async fn http_delete_asset_book(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_asset_book(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Deleted", "Asset book deleted"))
}

async fn http_create_asset_cwip(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateCWIP>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .new_cwip(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(
        asset,
        "Capital Work In Progress created",
    ))
}

async fn http_list_asset_cwip(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service.list_cwip(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(
        list,
        "Capital Work In Progress fetched",
    ))
}

async fn http_get_asset_cwip(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_cwip(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Capital Work In Progress fetched",
    ))
}

async fn http_update_asset_cwip(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateCWIP>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_cwip(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Capital Work In Progress updated",
    ))
}

async fn http_delete_asset_cwip(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_cwip(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        "Deleted",
        "Capital Work In Progress deleted",
    ))
}

async fn http_capitalize_cwip(
    State(_state): State<Arc<AppState>>,
    Path(cwip_uuid): Path<Uuid>,
    Path(asset_uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .capitalize_cwip(&user.tenant_pool, cwip_uuid, asset_uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        "Capitalized",
        "Capital Work In Progress deleted",
    ))
}

async fn http_compute_depreciation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<AssetDepreciation>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .compute_depreciation_and_post(
            &user.tenant_pool,
            user.user_id,
            payload.period_date,
            payload.asset_id,
        )
        .await?;
    Ok(ApiResponse::created(asset, "Depreciation computed"))
}

async fn http_run_depreciation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<AssetDepreciation>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .run_depreciation(&user.tenant_pool, payload.period_date, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Depreciation run"))
}

async fn http_get_depr_schedule(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_depreciation_schedule(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Depreciation schedule fetched"))
}

async fn http_post_depr_to_gl(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .post_depreciation_to_gl(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Posted depreciation to journal",
    ))
}
async fn http_create_asset_component(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetComponent>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .create_asset_component(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Asset component created"))
}

async fn http_list_asset_component(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_asset_components(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Asset components fetched"))
}

async fn http_get_asset_component(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_asset_component(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Asset component fetched"))
}

async fn http_update_asset_component(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetComponent>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_asset_component(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Asset component updated"))
}

async fn http_delete_asset_component(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_asset_component(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Deleted", "Asset component deleted"))
}

async fn http_create_insurance(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetInsurance>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .create_insurance(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Insurance created"))
}

async fn http_list_insurance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_insurance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Insurance fetched"))
}

async fn http_get_insurance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_insurance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Insurance fetched"))
}

async fn http_update_insurance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetInsurance>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_insurance(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Insurance updated"))
}

async fn http_delete_insurance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_insurance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        "Deleted",
        "Insurance schedule deleted",
    ))
}

async fn http_transfer_asset(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetTransfer>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .transfer_asset(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Asset transfered"))
}

async fn http_transfer_history(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .asset_transfer_history(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Asset transfer history fetched",
    ))
}

async fn http_transaction_history(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .asset_transaction_history(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Asset transaction history fetched",
    ))
}

async fn http_create_maintenance(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetMaintenance>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .create_maintenance(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(asset, "Maintenance created"))
}

async fn http_list_maintenance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_maintenance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Maintenance fetched"))
}

async fn http_get_maintenance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .get_maintenance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Maintenance fetched"))
}

async fn http_update_maintenance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetMaintenance>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .update_maintenance(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Maintenance updated"))
}

async fn http_delete_maintenance(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    service
        .delete_maintenance(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Asset Deleted", "Maintenance deleted"))
}

async fn http_create_revaluation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetRevaluation>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .create_revaluation(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Revaluation created"))
}

async fn http_revaluation_history(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .asset_revaluation_history(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(asset, "Revaluation history fetched"))
}

async fn http_dispose_asset(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateAssetDisposal>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .dispose_asset(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(asset, "Asset disposed"))
}

async fn http_disposal_history(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let asset = service
        .disposal_history(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        asset,
        "Asset disposal history fetched",
    ))
}
