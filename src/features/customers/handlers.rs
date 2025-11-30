use axum::{
    Extension, Router,
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
};
use sqlx::{FromRow, types::JsonValue};
use std::sync::Arc;

use crate::{
    features::customers::{repository::PostgresCustomerRepo, service::CustomerService},
    infrastructure::errors::AppError,
    infrastructure::responses::ApiResponse,
    middleware::auth::AuthenticatedTenant,
    models::{customers::CreateCustomer, invoice::CreateInvoice},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_customer))
        .route("/list", get(list_customers))
        .route("/get/{id}", get(get_customer))
        .route("/update/{id}", patch(update_customer))
        .route("/delete/{id}", delete(delete_customer))
        .route("/create/{cid}/invoices", post(create_invoice))
}

async fn create_customer(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service
        .create(&user.tenant_pool, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(cust, "Customer created"))
}

async fn list_customers(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let list = service.list(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Customers fetched"))
}

async fn get_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service.get(&user.tenant_pool, id, user.user_id).await?;
    Ok(ApiResponse::success(cust, "Customer fetched"))
}

async fn update_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service
        .update(&user.tenant_pool, id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::success(cust, "Customer updated"))
}

async fn delete_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    service.delete(&user.tenant_pool, id, user.user_id).await?;
    Ok(ApiResponse::success((), "Customer deleted"))
}

// Handle Invoices
#[derive(sqlx::Type, serde::Serialize, FromRow)]
#[sqlx(type_name = "bigint")]
pub struct InvoiceId {
    pub id: i64,
}

async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateInvoice>,
) -> Result<Response, AppError> {
    let rows = sqlx::query_as::<_, InvoiceId>(
        r#"
        SELECT receivables.create_invoice_and_post_gl(
            $1, $2, $3, $4, $5, $6, $7
        ) AS id
        "#,
    )
    .bind(&payload.invoice_number)
    .bind(payload.customer_id)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(&payload.total)
    .bind(payload.items.as_ref().map(|v| JsonValue::from(v.clone())))
    .bind(user.user_id) // assuming system.users.id is BIGINT
    .fetch_one(&state.master_pool)
    .await?;

    Ok(ApiResponse::created(rows, "Invoice created & GL posted"))
}
