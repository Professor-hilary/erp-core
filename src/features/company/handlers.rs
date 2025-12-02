// src/features/company/handlers.rs
use crate::{
    features::company::services::CompanyService,
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    models::{
        company::{Company, CreateCompanyDto, UpdateCompanyDto},
    },
    state::AppState,
};
use axum::{
    Extension, Json, Router, extract::{Path, State}, response::IntoResponse, routing::{delete, get, post, put}
};
use std::sync::Arc;
use uuid::Uuid;

// Bring the trait into scope so we can call methods on the repo
use crate::features::company::repository::{CompanyRepository, PostgresCompanyRepository};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_company))
        .route("/list", get(list_companies))
        .route("/get/{id}", get(get_company))
        .route("/update/{id}", put(update_company))
        .route("/delete/{id}", delete(delete_company))
}

/// POST /companies
async fn create_company(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateCompanyDto>,
) -> Result</* Json<(Company, String)> */impl IntoResponse, AppError> {
    let (company, token_str) =
        CompanyService::create_company(state.clone(), user.user_id, payload).await?;

    Ok(ApiResponse::created_with_token(
        company,
        token_str,
        "Company Created Successfully",
        "company"
    ))
}

/// GET /companies (user's companies)
async fn list_companies(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<AuthenticatedTenant>,
) -> Result<Json<Vec<Company>>, AppError> {
    let companies: Vec<Company> =
        CompanyService::list_user_companies(state.clone(), tenant.user_id)
            .await
            .map_err(|_| AppError::Internal("Failed to fetch companies".into()))?;
    Ok(Json(companies))
}

/// GET /companies/:id
async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Extension(_tenant): Extension<AuthenticatedTenant>,
) -> Result<Json<Company>, AppError> {
    let repo = PostgresCompanyRepository;
    let company = repo
        .find_by_id(&state.master_pool, company_id)
        .await
        .map_err(|_| AppError::Internal("Failed to fetch companies".into()))?
        .ok_or(AppError::NotFound("Company not found".into()))?;
    Ok(Json(company))
}

/// PUT /companies/:id
async fn update_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Extension(tenant): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateCompanyDto>,
) -> Result<Json<Company>, AppError> {
    let company: Company =
        CompanyService::update_company(state.clone(), tenant.user_id, company_id, payload)
            .await
            .map_err(|_| AppError::Internal("Failed to fetch companies".into()))?;
    Ok(Json(company))
}

/// DELETE /companies/:id (soft delete)
async fn delete_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Extension(tenant): Extension<AuthenticatedTenant>,
) -> Result<Json<serde_json::Value>, AppError> {
    CompanyService::delete_company(state.clone(), tenant.user_id, company_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Company deleted successfully (soft delete)"
    })))
}
