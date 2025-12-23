// src/features/company/handlers.rs
use crate::{
    features::{
        auth::{AuthService, repository::PostgresUserRepo},
        company::services::CompanyService,
    },
    infrastructure::{errors::AppError, responses::ApiResponse},
    middleware::auth::{AuthenticatedTenant, AuthenticatedUser},
    models::company::{Company, CreateCompanyDto, UpdateCompanyDto},
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

// Bring the trait into scope so we can call methods on the repo
use crate::features::company::repository::{CompanyRepository, PostgresCompanyRepository};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create_company))
        .route("/switch-company", post(switch_company))
        .route("/list", get(list_companies))
        .route("/get/{id}", get(get_company))
        .route("/update/{id}", put(update_company))
        .route("/delete/{id}", delete(delete_company))
}

/// POST /companies/create
async fn create_company(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateCompanyDto>,
) -> Result<impl IntoResponse, AppError> {
    let (company, token_str) =
        CompanyService::create_company(state.clone(), user.user_id, payload).await?;

    Ok(ApiResponse::created_with_token(
        company,
        token_str,
        "Company Created Successfully",
    ))
}

/// GET /companies/list (user's companies)
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

/// GET /companies/get/:id
async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(company_id): Path<Uuid>,
    Extension(_tenant): Extension<AuthenticatedTenant>,
) -> Result<Json<Company>, AppError> {
    let repo: PostgresCompanyRepository = PostgresCompanyRepository;
    let company: Company = repo
        .find_by_id(&state.master_pool, company_id)
        .await
        .map_err(|_| AppError::Internal("Failed to fetch companies".into()))?
        .ok_or(AppError::NotFound("Company not found".into()))?;
    Ok(Json(company))
}

/// PUT /companies/update/:id
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

/// DELETE /companies/delete/:id (soft delete)
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

// POST /api/companies/switch-company { "company_id": "..." }
async fn switch_company(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let repo: PostgresUserRepo = PostgresUserRepo::new(state.master_pool.clone());

    let company_id: Uuid = payload["company_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::BadRequest("Invalid company_id".into()))?;

    let company: Company = PostgresCompanyRepository
        .find_by_id(&state.master_pool, company_id)
        .await
        .map_err(|_| AppError::Internal("Failed to fetch companies".into()))?
        .ok_or(AppError::NotFound("Company not found".into()))?;

    let service: AuthService<PostgresUserRepo> = AuthService::new(repo, state.clone());
    let new_token: String = service
        .switch_company(user.user_id, company_id, state)
        .await?;

    Ok(ApiResponse::created_with_token(
        company,
        new_token,
        "Company switched successfully",
    ))
}
