use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::customers::{repository::PostgresCustomerRepo, services::CustomerService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::customers::{ApplyPayment, CreateCustomer, CreateInvoice, Customer, PostInvoice},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/customer", post(http_create_customer))
        .route("/create/invoice", post(http_create_invoice))
        .route("/post-invoice", post(http_post_invoice))
        .route("/get/{uuid}", get(http_get_customer))
        .route("/list/customers", get(http_list_customers))
        .route("/list/invoices/{uuid}", get(http_list_invoices))
        .route("/pay-invoice", get(http_apply_invoice_payment))
        .route("/update/{uuid}", patch(http_update_customer))
        .route("/delete/{uuid}", delete(http_delete_customer))
}

async fn http_create_customer(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customer: Customer = service
        .create(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(customer, "Customer created"))
}

async fn http_list_customers(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customers: Vec<Customer> = service.list(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(customers, "Customers fetched"))
}

async fn http_get_customer(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customer: Customer = service.get(&user.tenant_pool, uuid, user.user_id).await?;
    Ok(ApiResponse::success(customer, "Customer fetched"))
}

async fn http_update_customer(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customer: Customer = service
        .update(&user.tenant_pool, uuid, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(customer, "Customer updated"))
}

async fn http_delete_customer(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customer_id = service
        .delete(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success(
        customer_id,
        "Customer deleted successfully",
    ))
}

async fn http_create_invoice(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateInvoice>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let invoice = service.create_invoice(&user.tenant_pool, &payload).await?;
    Ok(ApiResponse::success(
        invoice,
        "Customer invoice created successfully",
    ))
}

async fn http_post_invoice(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PostInvoice>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let invoice = service
        .post_invoice(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(invoice, "Customer invoice posted"))
}

async fn http_apply_invoice_payment(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<ApplyPayment>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let payment = service.apply_payment(&user.tenant_pool, payload).await?;
    Ok(ApiResponse::success(
        payment,
        "Customer invoice payment applied",
    ))
}

async fn http_list_invoices(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let invoices = service
        .list_customer_invoice(&user.tenant_pool, uuid)
        .await?;
    Ok(ApiResponse::success(invoices, "Customer invoices found"))
}
