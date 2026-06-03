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
        .route("/asset/list", post(http_list_assets))
        .route("/asset/get/{uuid}", get(http_get_asset))
        .route("/asset/delete", delete(http_delete_asset))
        .route("/asset-class/create", post(http_create_asset_class))
        .route("/asset-class/update", patch(http_update_asset_class))
        .route("/asset-class/list", post(http_list_asset_classes))
        .route("/asset-class/get/{uuid}", get(http_get_asset_class))
        .route("/asset-class/delete", delete(http_delete_asset_class))
        .route("/asset-book/create", post(http_create_asset_book))
        .route("/asset-book/update", patch(http_update_asset_book))
        .route("/asset-book/list", post(http_list_asset_books))
        .route("/asset-book/get/{uuid}", get(http_get_asset_book))
        .route("/asset-book/delete", delete(http_delete_asset_book))
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
        .list_asset_class(&user.tenant_pool, user.user_id)
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
    AppJson(payload): AppJson<CreateAssetClass>,
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
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresFixedAssetRepository::new();
    let service = FixedAssetService::new(repo);
    let list = service
        .list_asset_books(&user.tenant_pool, user.user_id)
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
    AppJson(payload): AppJson<CreateAssetClass>,
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
