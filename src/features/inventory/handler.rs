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
    features::inventory::{repository::PostgresInventoryRepo, service::InventoryService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::item::{CreateItem, Item, PostPurchase, PostSale},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_item))
        .route("/list", get(list_items))
        .route("/get/{uuid}", get(get_item))
        .route("/update/{uuid}", patch(update_item))
        .route("/items/{uuid}", delete(delete_item))
        .route("/purchases", post(post_purchase))
        .route("/sales", post(post_sale))
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

async fn list_items(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    let list: Vec<Item> = service.list_items(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Items fetched"))
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

async fn post_purchase(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostPurchase>,
) -> Result<Response, AppError> {
    let repo: PostgresInventoryRepo = PostgresInventoryRepo::new();
    let service: InventoryService<PostgresInventoryRepo> = InventoryService::new(repo);
    service
        .post_purchase(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(
        (),
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
