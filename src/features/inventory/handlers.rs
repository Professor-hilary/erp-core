// src/features/inventory/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::inventory::{repository::PostgresInventoryRepo, services::InventoryService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::inventory::{
        CreateItem, CreateItemCategory, CreateWarehouse, Item, ItemCategory, PostPurchase,
        PostSale, Warehouse,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/item", post(http_create_item))
        .route("/create/category", post(http_create_item_category))
        .route("/create/warehouse", post(http_create_warehouse))
        .route("/create/cash-purchase", post(http_cash_purchase))
        .route("/create/cash-sale", post(http_post_sale))
        .route("/list/inventory", get(http_list_items))
        .route("/list/categories", get(http_list_item_categories))
        .route("/list/warehouses", get(http_list_warehouses))
        .route("/get/item/{uuid}", get(http_get_item))
        .route("/get/category/{uuid}", get(http_get_item_category))
        .route("/get/warehouse/{uuid}", get(http_get_warehouse))
        .route("/update/item/{uuid}", patch(http_update_item))
        .route("/update/category/{uuid}", patch(http_update_item_category))
        .route("/update/warehouse/{uuid}", patch(http_update_warehouse))
        .route("/delete/item/{uuid}", delete(http_delete_item))
        .route("/delete/category/{uuid}", delete(http_delete_item_category))
        .route("/delete/warehouse/{uuid}", delete(http_delete_warehouse))
}

async fn http_create_item(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: Item = service
        .create_item(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(item, "Item created"))
}

async fn http_create_warehouse(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateWarehouse>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let category: Warehouse = service
        .create_warehouse(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(category, "Warehouse created"))
}

async fn http_create_item_category(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateItemCategory>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let category: ItemCategory = service
        .create_item_category(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(category, "Item category created"))
}

async fn http_list_items(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let list: Vec<Item> = service.list_items(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Items fetched"))
}

async fn http_list_item_categories(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let list: Vec<ItemCategory> = service
        .list_item_categories(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Item categories fetched"))
}

async fn http_list_warehouses(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let list: Vec<Warehouse> = service
        .list_warehouse(&user.tenant_pool, user.user_id)
        .await?;
    Ok(ApiResponse::success(list, "Warehouses fetched"))
}

async fn http_get_item(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: Item = service
        .get_item(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(item, "Item fetched"))
}

async fn http_get_item_category(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: ItemCategory = service
        .get_item_category(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(item, "Item category fetched"))
}

async fn http_get_warehouse(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: Warehouse = service
        .get_warehouse(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(item, "Warehouse fetched"))
}

async fn http_update_item(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: Item = service
        .update_item(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(item, "Item updated"))
}

async fn http_update_item_category(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateItemCategory>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let item: ItemCategory = service
        .update_item_category(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(item, "Item category updated"))
}

async fn http_update_warehouse(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateWarehouse>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let warehouse: Warehouse = service
        .update_warehouse(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(warehouse, "Warehouse updated"))
}

async fn http_delete_item(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    service
        .delete_item(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Item deleted"))
}

async fn http_delete_item_category(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    service
        .delete_item_category(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Item category deleted"))
}

async fn http_delete_warehouse(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    service
        .delete_warehouse(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Warehouse deleted"))
}

async fn http_cash_purchase(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostPurchase>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let purchase = service
        .post_purchase(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(
        purchase,
        "Purchase posted to inventory and GL",
    ))
}

async fn http_post_sale(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostSale>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    service
        .post_sale(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success((), "Sale posted to inventory and GL"))
}
