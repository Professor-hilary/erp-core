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
        .route("/create/item", post(create_item))
        .route("/create/category", post(create_item_category))
        .route("/create/warehouse", post(create_warehouse))
        .route("/list/items", get(list_items))
        .route("/list/categories", get(list_item_categories))
        .route("/list/warehouses", get(list_warehouses))
        .route("/get/item/{uuid}", get(get_item))
        .route("/get/category/{uuid}", get(get_item_category))
        .route("/get/warehouse/{uuid}", get(get_warehouse))
        .route("/update/item/{uuid}", patch(update_item))
        .route("/update/category/{uuid}", patch(update_item_category))
        .route("/update/warehouse/{uuid}", patch(update_warehouse))
        .route("/delete/item/{uuid}", delete(delete_item))
        .route("/delete/category/{uuid}", delete(delete_item_category))
        .route("/delete/warehouse/{uuid}", delete(delete_warehouse))
        .route("/purchase", post(post_purchase))
        .route("/sale", post(post_sale))
}

async fn create_item(
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

async fn create_warehouse(
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

async fn create_item_category(
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

async fn list_items(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let list: Vec<Item> = service.list_items(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Items fetched"))
}

async fn list_item_categories(
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

async fn list_warehouses(
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

async fn get_item(
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

async fn get_item_category(
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

async fn get_warehouse(
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

async fn update_item(
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

async fn update_item_category(
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

async fn update_warehouse(
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

async fn delete_item(
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

async fn delete_item_category(
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

async fn delete_warehouse(
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

async fn post_purchase(
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

async fn post_sale(
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
