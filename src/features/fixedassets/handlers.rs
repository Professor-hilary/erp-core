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
        .route("/depreciation/compute", patch(http_compute_depreciation))
        .route("/depreciation/run", get(http_run_depreciation))
        .route("/depreciation/schedule/{uuid}", get(http_get_depr_schedule))
        .route("/depreciation/post/{uuid}", post(http_post_depr_to_gl))
        .route("/component/create", post(http_create_asset_component))
        .route("/component/update", patch(http_update_asset_component))
        .route("/component/list", get(http_list_asset_component))
        .route("/component/get/{uuid}", get(http_get_asset_component))
        .route("/component/delete", delete(http_delete_asset_component))
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
    Ok(ApiResponse::success(asset, "Vendor updated"))
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
    Ok(ApiResponse::created(asset, "Asset created"))
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
    Ok(ApiResponse::success(list, "Assets fetched"))
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
    Ok(ApiResponse::success(asset, "Asset fetched"))
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
    Ok(ApiResponse::success(asset, "Vendor updated"))
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
    Ok(ApiResponse::success("Asset Deleted", "Asset deleted"))
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
    Ok(ApiResponse::success("Asset Deleted", "Asset book deleted"))
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
        "Asset Deleted",
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
        "Asset Deleted",
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
        .compute_depreciation(
            &user.tenant_pool,
            user.user_id,
            payload.period_date,
            payload.asset_id,
        )
        .await?;
    Ok(ApiResponse::created(
        asset,
        "Capital Work In Progress created",
    ))
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
    Ok(ApiResponse::success(
        list,
        "Capital Work In Progress fetched",
    ))
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
    Ok(ApiResponse::success(
        asset,
        "Capital Work In Progress fetched",
    ))
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
        "Capital Work In Progress updated",
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
    Ok(ApiResponse::created(
        asset,
        "Capital Work In Progress created",
    ))
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
    Ok(ApiResponse::success(
        list,
        "Capital Work In Progress fetched",
    ))
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
    Ok(ApiResponse::success(
        asset,
        "Capital Work In Progress fetched",
    ))
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
    Ok(ApiResponse::success(
        asset,
        "Capital Work In Progress updated",
    ))
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
    Ok(ApiResponse::success(
        "Asset Deleted",
        "Capital Work In Progress deleted",
    ))
}
