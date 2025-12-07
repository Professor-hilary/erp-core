use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use sqlx::FromRow;

use sqlx::types::JsonValue;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::vendors::{repository::PostgresVendorRepo, service::VendorService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::vendor::{CreateBill, CreateVendor, Vendor},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_vendor))
        .route("/list", get(list_vendors))
        .route("/get/{uuid}", get(get_vendor))
        .route("/update/{uuid}", patch(update_vendor))
        .route("/delete/{uuid}", delete(delete_vendor))
        .route("/create/{cid}/bills", post(create_bill))
}

async fn create_vendor(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let vendor: Vendor = service
        .create(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(vendor, "Vendor created"))
}

async fn list_vendors(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let list: Vec<Vendor> = service.list(&user.tenant_pool, user.user_id).await?;
    // let total_vendors = list.len();
    Ok(ApiResponse::success(list, "Vendors fetched"))
}

async fn get_vendor(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let vendor: Vendor = service.get(&user.tenant_pool, uuid, user.user_id).await?;
    Ok(ApiResponse::success(vendor, "Vendor fetched"))
}

async fn update_vendor(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateVendor>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let vendor: Vendor = service
        .update(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(vendor, "Vendor updated"))
}

async fn delete_vendor(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    service
        .delete(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success("Vendor Deleted", "Vendor deleted"))
}

// Handle Bills
#[derive(sqlx::Type, serde::Serialize, FromRow)]
#[sqlx(type_name = "bigint")]
pub struct BillId {
    pub uuid: Uuid,
}

async fn create_bill(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateBill>,
) -> Result<Response, AppError> {
    let rows: BillId = sqlx::query_as::<_, BillId>(
        r#"
        SELECT payables.create_bill_and_post_gl(
            $1, $2, $3, $4, $5, $6, $7
        ) AS uuid
        "#,
    )
    .bind(&payload.bill_number)
    .bind(payload.vendor_id)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(&payload.total)
    .bind(payload.items.as_ref().map(|v| JsonValue::from(v.clone())))
    .bind(user.user_id) // assuming system.users.uuid is BIGINT
    .fetch_one(&_state.master_pool)
    .await?;

    Ok(ApiResponse::created(rows, "Bill created & GL posted"))
}
