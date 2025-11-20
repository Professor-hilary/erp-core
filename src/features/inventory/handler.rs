// src/features/inventory/handlers.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;

use crate::{
    errors::AppError,
    features::inventory::{repository::PostgresInventoryRepo, service::InventoryService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::{
        inventory_movement::{PostPurchase, PostSale},
        item::CreateItem,
    },
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/items", post(create_item))
        .route("/items", get(list_items))
        .route("/items/{id}", get(get_item))
        .route("/items/{id}", patch(update_item))
        .route("/items/{id}", delete(delete_item))
        .route("/purchases", post(post_purchase))
        .route("/sales", post(post_sale))
}

async fn create_item(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    let item = service
        .create_item(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(item, "Item created"))
}

async fn list_items(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    let list = service.list_items(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Items fetched"))
}

async fn get_item(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    let item = service
        .get_item(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success(item, "Item fetched"))
}

async fn update_item(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    let item = service
        .update_item(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(item, "Item updated"))
}

async fn delete_item(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    service
        .delete_item(&user.tenant_pool, id, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Item deleted"))
}

async fn post_purchase(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<PostPurchase>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
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
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<PostSale>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new();
    let service = InventoryService::new(repo);
    service
        .post_sale(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success((), "Sale posted to inventory and GL"))
}
