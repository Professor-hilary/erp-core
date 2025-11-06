use axum::{
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use sqlx::FromRow;

use std::sync::Arc;
use sqlx::types::JsonValue;

use crate::{
    errors::AppError,
    features::vendors::{repository::PostgresVendorRepo, service::VendorService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::{bills::CreateBill, vendor::CreateVendor},
    routes::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/vendors", post(create_vendor))
        .route("/vendors", get(list_vendors))
        .route("/vendors/{id}", get(get_vendor))
        .route("/vendors/{id}", patch(update_vendor))
        .route("/vendors/{id}", delete(delete_vendor))
        .route("/vendors/{cid}/bills", post(create_bill))
}

async fn create_vendor(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new(state.pool.clone());
    let svc = VendorService::new(repo);
    let cust = svc.create(user.0, &payload).await?;
    Ok(ApiResponse::created(cust, "Vendor created"))
}

async fn list_vendors(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new(state.pool.clone());
    let svc = VendorService::new(repo);
    let list = svc.list(user.0).await?;
    Ok(ApiResponse::success(list, "Vendors fetched"))
}

async fn get_vendor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new(state.pool.clone());
    let svc = VendorService::new(repo);
    let cust = svc.get(id, user.0).await?;
    Ok(ApiResponse::success(cust, "Vendor fetched"))
}

async fn update_vendor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new(state.pool.clone());
    let svc = VendorService::new(repo);
    let cust = svc.update(id, user.0, &payload).await?;
    Ok(ApiResponse::success(cust, "Vendor updated"))
}

async fn delete_vendor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new(state.pool.clone());
    let svc = VendorService::new(repo);
    svc.delete(id, user.0).await?;
    Ok(ApiResponse::success((), "Vendor deleted"))
}

// Handle Bills
#[derive(sqlx::Type, serde::Serialize, FromRow)]
#[sqlx(type_name = "bigint")]
pub struct BillId {
    pub id: i64,
}

async fn create_bill(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateBill>,
) -> Result<Response, AppError> {
    let rows = sqlx::query_as::<_, BillId>(
        r#"
        SELECT payables.create_bill_and_post_gl(
            $1, $2, $3, $4, $5, $6, $7
        ) AS id
        "#,
    )
    .bind(&payload.bill_number)
    .bind(payload.vendor_id)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(&payload.total)
    .bind(payload.items.as_ref().map(|v| JsonValue::from(v.clone())))
    .bind(user.0) // assuming system.users.id is BIGINT
    .fetch_one(&state.pool)
    .await?;

    Ok(ApiResponse::created(rows, "Bill created & GL posted"))
}
