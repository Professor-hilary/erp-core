use axum::{
    extract::{Json, Path, State},
    response::Response,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use std::sync::Arc;
use sqlx::{FromRow, types::JsonValue};

use crate::{
    errors::AppError,
    features::customers::{repository::PostgresCustomerRepo, service::CustomerService},
    infrastructure::responses::ApiResponse,
    middleware::Authenticated,
    models::{customers::CreateCustomer, invoice::CreateInvoice},
    state::AppState,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/customers", post(create_customer))
        .route("/customers", get(list_customers))
        .route("/customers/{id}", get(get_customer))
        .route("/customers/{id}", patch(update_customer))
        .route("/customers/{id}", delete(delete_customer))
        .route("/customers/{cid}/invoices", post(create_invoice))
}

async fn create_customer(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service.create(&user.tenant_pool,user.user_id, &payload).await?;
    Ok(ApiResponse::created(cust, "Customer created"))
}

async fn list_customers(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let list = service.list(&user.tenant_pool, user.user_id).await?;
    Ok(ApiResponse::success(list, "Customers fetched"))
}

async fn get_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service.get(&user.tenant_pool,id, user.user_id).await?;
    Ok(ApiResponse::success(cust, "Customer fetched"))
}

async fn update_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    let cust = service.update(&user.tenant_pool,id, user.user_id, &payload).await?;
    Ok(ApiResponse::success(cust, "Customer updated"))
}

async fn delete_customer(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Extension(user): Extension<Authenticated>,
) -> Result<Response, AppError> {
    let repo = PostgresCustomerRepo::new();
    let service = CustomerService::new(repo);
    service.delete(&user.tenant_pool,id, user.user_id).await?;
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
    Extension(user): Extension<Authenticated>,
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
