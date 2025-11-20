// src/features/company/handlers.rs
use crate::{
    features::company::services::CompanyService,
    //errors::AppError,
    infrastructure::responses::AppError,
    middleware::Authenticated,
    models::company::{Company, CreateCompanyDto, UpdateCompanyDto},
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

// Bring the trait into scope so we can call methods on the repo
use crate::features::company::repository::{CompanyRepository, PostgresCompanyRepository};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/companies", post(create_company))
        .route("/companies", get(list_companies))
        .route("/companies/:id", get(get_company))
        .route("/companies/:id", put(update_company))
        .route("/companies/:id", delete(delete_company))
}

/// POST /companies
async fn create_company(
    State(state): State<Arc<AppState>>,
    Authenticated { user_id, .. }: Authenticated,
    Json(payload): Json<CreateCompanyDto>,
) -> Result<Json<Company>, AppError> {
    let company: Company = CompanyService::create_company(state.clone(), user_id, payload).await?;
    Ok(Json(company))
}

/// GET /companies (user's companies)
async fn list_companies(
    State(state): State<Arc<AppState>>,
    Authenticated { user_id, .. }: Authenticated,
) -> Result<Json<Vec<Company>>, AppError> {
    let companies: Vec<Company> = CompanyService::list_user_companies(state.clone(), user_id)
        .await
        .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;
    Ok(Json(companies))
}

/// GET /companies/:id
async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Authenticated { user_id: _, .. }: Authenticated,
) -> Result<Json<Company>, AppError> {
    let repo = PostgresCompanyRepository;
    let company = repo
        .find_by_id(&state.master_pool, company_id)
        .await
        .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(company))
}

/// PUT /companies/:id
async fn update_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Authenticated { user_id, .. }: Authenticated,
    Json(payload): Json<UpdateCompanyDto>,
) -> Result<Json<Company>, AppError> {
    let company: Company =
        CompanyService::update_company(state.clone(), user_id, company_id, payload)
            .await
            .map_err(|_| AppError::Internal("Failed to assign admin role".into()))?;
    Ok(Json(company))
}

/// DELETE /companies/:id (soft delete)
async fn delete_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Authenticated { user_id, .. }: Authenticated,
) -> Result<Json<serde_json::Value>, AppError> {
    CompanyService::delete_company(state.clone(), user_id, company_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Company deleted successfully (soft delete)"
    })))
}
