// src/features/company/handlers.rs
use crate::{
    features::{
        auth::{AuthService, repository::PostgresUserRepo},
        company::services::CompanyService,
    },
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
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
        .route("/add-member", post(add_member))
        .route("/list", get(list_companies))
        .route("/get/{id}", get(get_company))
        .route("/update/{id}", put(update_company))
        .route("/delete/{id}", delete(delete_company))
}

/// POST /companies/create
async fn create_company(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    AppJson(payload): AppJson<CreateCompanyDto>,
) -> Result<impl IntoResponse, AppError> {
    let (company, access, refresh) =
        CompanyService::create_company(state.clone(), user.user_id, payload).await?;

    Ok(ApiResponse::created_with_tokens(
        company,
        access,
        refresh,
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
    Extension(_tenant): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<UpdateCompanyDto>,
) -> Result<Json<Company>, AppError> {
    let company: Company = CompanyService::update_company(state.clone(), company_id, payload)
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
    AppJson(payload): AppJson<serde_json::Value>,
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
    let (access, refresh) = service
        .switch_company(user.user_id, company_id, state)
        .await?;

    Ok(ApiResponse::created_with_tokens(
        company,
        access,
        refresh,
        "Company switched successfully",
    ))
}

/// POST /api/companies/add-member
/// Link an existing registered user to this company (master.user_companies).
/// Tenant RBAC membership is assigned separately via /api/rbac/memberships.
async fn add_member(
    State(state): State<Arc<AppState>>,
    Extension(tenant): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<crate::features::rbac::Actor>,
    AppJson(payload): AppJson<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Only tenant admins / settings managers may link users
    actor.require_module(crate::features::rbac::Module::Settings)?;
    actor.require("settings.manage_users")?;

    let user_uuid = payload
        .get("user_uuid")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("user_uuid required".into()))?;

    let repo = PostgresCompanyRepository;
    repo.assign_user_as_member(&state.master_pool, user_uuid, tenant.company_id)
        .await?;

    Ok(ApiResponse::success(
        serde_json::json!({
            "user_uuid": user_uuid,
            "company_id": tenant.company_id,
            "role": "member"
        }),
        "User linked to company",
    ))
}
