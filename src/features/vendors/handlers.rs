use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};

use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::vendors::{repository::PostgresVendorRepo, services::VendorService},
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::vendor::{ApplyPayment, Bill, CreateBill, CreateVendor, Payment, PostBill, Vendor},
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
    AppJson(payload): AppJson<CreateVendor>,
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
    AppJson(payload): AppJson<CreateVendor>,
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
    AppJson(payload): AppJson<CreateBill>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let bill: Bill = service.create_bill(&user.tenant_pool, &payload).await?;

    Ok(ApiResponse::created(bill, "Bill created successfully"))
}

async fn http_post_bill(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<PostBill>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let bill_id: i64 = service
        .post_bill(&user.tenant_pool, user.user_id, &payload)
        .await?;

    Ok(ApiResponse::success(
        format!("Bill serial id: {bill_id}"),
        "Bill posted successfully",
    ))
}

async fn http_apply_bill_payment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyPayment>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let payment: Payment = service.apply_payment(&user.tenant_pool, payload).await?;
    Ok(ApiResponse::success(
        payment,
        "Bill payment applied successfully",
    ))
}

async fn http_list_vendor_bills(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresVendorRepo = PostgresVendorRepo::new();
    let service: VendorService<PostgresVendorRepo> = VendorService::new(repo);
    let bills: Vec<Bill> = service.list_vendor_bills(&user.tenant_pool, uuid).await?;
    Ok(ApiResponse::success(bills, "Vendor bills found"))
}
