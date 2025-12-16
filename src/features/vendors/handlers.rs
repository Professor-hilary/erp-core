use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::vendors::{repository::PostgresVendorRepo, services::VendorService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::vendor::{ApplyPayment, CreateBill, CreateVendor, PostBill, Vendor},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/vendor", post(http_create_vendor))
        .route("/create/bill", post(http_create_bill))
        .route("/post-bill", post(http_post_bill))
        .route("/get/{uuid}", get(http_get_vendor))
        .route("/list/vendors", get(http_list_vendors))
        .route("/list/vendor-bills/{uuid}", get(http_list_vendor_bills))
        .route("/apply-bill-pay", get(http_apply_bill_payment))
        .route("/update/{uuid}", patch(http_update_vendor))
        .route("/delete/{uuid}", delete(http_delete_vendor))
}

async fn http_create_vendor(
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

async fn http_list_vendors(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let list: Vec<Vendor> = service.list(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Vendors fetched"))
}

async fn http_get_vendor(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let vendor: Vendor = service.get(&user.tenant_pool, uuid, user.user_id).await?;
    Ok(ApiResponse::success(vendor, "Vendor fetched"))
}

async fn http_update_vendor(
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

async fn http_delete_vendor(
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

async fn http_create_bill(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateBill>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    service.create_bill(&user.tenant_pool, &payload).await?;
    Ok(ApiResponse::success("Created Bill", "Vendor deleted"))
}

async fn http_post_bill(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostBill>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    service
        .post_bill(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success("Bill Posted", "Vendor deleted"))
}

async fn http_apply_bill_payment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<ApplyPayment>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    service.apply_payment(&user.tenant_pool, payload).await?;
    Ok(ApiResponse::success("Bill Posted", "Vendor deleted"))
}

async fn http_list_vendor_bills(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    service.list_vendor_bills(&user.tenant_pool, uuid).await?;
    Ok(ApiResponse::success("Vendor Bills Found", "Vendor deleted"))
}
