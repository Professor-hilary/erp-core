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
    errors::AppError, features::vendors::{repository::PostgresVendorRepo, service::VendorService}, infrastructure::responses::ApiResponse, middleware::auth::AuthenticatedTenant, models::{bills::CreateBill, vendor::CreateVendor}, state::AppState
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
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new();
    let svc = VendorService::new(repo);
    let cust = svc.create(&user.tenant_pool,user.user_id, &payload).await?;
    Ok(ApiResponse::created(cust, "Vendor created"))
}

async fn list_vendors(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new();
    let svc = VendorService::new(repo);
    let list = svc.list(&user.tenant_pool,user.user_id).await?;
    Ok(ApiResponse::success(list, "Vendors fetched"))
}

async fn get_vendor(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new();
    let svc = VendorService::new(repo);
    let cust = svc.get(&user.tenant_pool,id, user.user_id).await?;
    Ok(ApiResponse::success(cust, "Vendor fetched"))
}

async fn update_vendor(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new();
    let svc = VendorService::new(repo);
    let cust = svc.update(&user.tenant_pool,id, user.user_id, &payload).await?;
    Ok(ApiResponse::success(cust, "Vendor updated"))
}

async fn delete_vendor(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresVendorRepo::new();
    let svc = VendorService::new(repo);
    svc.delete(&user.tenant_pool,id, user.user_id).await?;
    Ok(ApiResponse::success((), "Vendor deleted"))
}

// Handle Bills
#[derive(sqlx::Type, serde::Serialize, FromRow)]
#[sqlx(type_name = "bigint")]
pub struct BillId {
    pub id: i64,
}

async fn create_bill(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
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
    .bind(user.user_id) // assuming system.users.id is BIGINT
    .fetch_one(&_state.master_pool)
    .await?;

    Ok(ApiResponse::created(rows, "Bill created & GL posted"))
}
