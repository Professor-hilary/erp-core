// src/features/inventory/handlers.rs
use axum::{
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use std::sync::Arc;

use crate::{
    errors::AppError,
    features::inventory::{repository::PostgresInventoryRepo, service::InventoryService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::{inventory_movement::{PostPurchase, PostSale}, item::{CreateItem}},
    routes::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/items", post(create_item))
        .route("/items", get(list_items))
        .route("/items/:id", get(get_item))
        .route("/items/:id", patch(update_item))
        .route("/items/:id", delete(delete_item))
        .route("/purchases", post(post_purchase))
        .route("/sales", post(post_sale))
}

async fn create_item(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    let item = svc.create_item(user.0, &payload).await?;
    Ok(ApiResponse::created(item, "Item created"))
}

async fn list_items(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    let list = svc.list_items(user.0).await?;
    Ok(ApiResponse::success(list, "Items fetched"))
}

async fn get_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    let item = svc.get_item(id, user.0).await?;
    Ok(ApiResponse::success(item, "Item fetched"))
}

async fn update_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateItem>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    let item = svc.update_item(id, user.0, &payload).await?;
    Ok(ApiResponse::success(item, "Item updated"))
}

async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    svc.delete_item(id, user.0).await?;
    Ok(ApiResponse::success((), "Item deleted"))
}

async fn post_purchase(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<PostPurchase>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    svc.post_purchase(user.0, &payload).await?;
    Ok(ApiResponse::success((), "Purchase posted to inventory and GL"))
}

async fn post_sale(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<PostSale>,
) -> Result<Response, AppError> {
    let repo = PostgresInventoryRepo::new(state.pool.clone());
    let svc = InventoryService::new(repo);
    svc.post_sale(user.0, &payload).await?;
    Ok(ApiResponse::success((), "Sale posted to inventory and GL"))
}