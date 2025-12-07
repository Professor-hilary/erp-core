use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use sqlx::{FromRow, types::JsonValue};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::customers::{repository::PostgresCustomerRepo, service::CustomerService},
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::customers::{CreateCustomer, CreateInvoice, Customer},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_customer))
        .route("/list", get(list_customers))
        .route("/get/{uuid}", get(get_customer))
        .route("/update/{uuid}", patch(update_customer))
        .route("/delete/{uuid}", delete(delete_customer))
        .route("/create/{cid}/invoices", post(create_invoice))
}

async fn create_customer(
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

async fn list_customers(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customers: Vec<Customer> = service.list(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(customers, "Customers fetched"))
}

async fn get_customer(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    let customer: Customer = service.get(&user.tenant_pool, uuid, user.user_id).await?;
    Ok(ApiResponse::success(customer, "Customer fetched"))
}

async fn update_customer(
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

async fn delete_customer(
    State(_state): State<Arc<AppState>>,
    Path(uuid): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo: PostgresCustomerRepo = PostgresCustomerRepo::new();
    let service: CustomerService<PostgresCustomerRepo> = CustomerService::new(repo);
    service
        .delete(&user.tenant_pool, uuid, user.user_id)
        .await?;
    Ok(ApiResponse::success((), "Customer deleted"))
}

// Handle Invoices
#[derive(sqlx::Type, serde::Serialize, FromRow)]
#[sqlx(type_name = "bigint")]
pub struct InvoiceId {
    pub uuid: Uuid,
}

async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateInvoice>,
) -> Result<Response, AppError> {
    let rows: InvoiceId = sqlx::query_as::<_, InvoiceId>(
        r#"
        SELECT receivables.create_invoice_and_post_gl(
            $1, $2, $3, $4, $5, $6, $7
        ) AS uuid
        "#,
    )
    .bind(&payload.invoice_number)
    .bind(payload.customer_id)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(&payload.total)
    .bind(payload.items.as_ref().map(|v| JsonValue::from(v.clone())))
    .bind(user.user_id) // assuming system.users.uuid is BIGINT
    .fetch_one(&state.master_pool)
    .await?;

    Ok(ApiResponse::created(rows, "Invoice created & GL posted"))
}
