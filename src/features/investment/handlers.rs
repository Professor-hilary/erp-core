use crate::{
    features::investment::{
        repository::PostgresInvestmentRepo,
        services::InvestmentService,
    },
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::investment::*,
    state::AppState,
};
use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/programs", post(create_program).get(list_programs))
        .route("/cases", post(create_case).get(list_cases))
        .route("/cases/{id}", get(get_case).put(update_case))
        .route("/cases/{id}/advance", post(advance_stage))
        .route(
            "/cases/{id}/scenarios",
            post(create_scenario).get(list_scenarios),
        )
        .route(
            "/scenarios/{scenario_id}/cashflows",
            post(add_cashflow).get(list_cashflows),
        )
        .route(
            "/cases/{case_id}/scenarios/{scenario_id}/recalculate",
            post(recalculate),
        )
        .route("/cases/{id}/risks", post(create_risk).get(list_risks))
        .route("/cases/{id}/post-audit", post(post_audit))
        .route("/portfolio", get(portfolio_board))
}

fn service() -> InvestmentService<PostgresInvestmentRepo> {
    InvestmentService::new(PostgresInvestmentRepo::new())
}

async fn create_program(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateInvestmentProgram>,
) -> Result<Response, AppError> {
    let row = service()
        .create_program(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Program created"))
}

async fn list_programs(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = service()
        .list_programs(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(rows, "Programs fetched"))
}

async fn create_case(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateInvestmentCase>,
) -> Result<Response, AppError> {
    let row = service()
        .create_case(
            &user.tenant_pool,
            user.company_id,
            user.user_id,
            &payload,
        )
        .await?;
    Ok(ApiResponse::created(row, "Investment case created"))
}

#[derive(Debug, Deserialize)]
pub struct ListCasesQuery {
    pub stage: Option<String>,
}

async fn list_cases(
    Extension(user): Extension<AuthenticatedTenant>,
    Query(q): Query<ListCasesQuery>,
) -> Result<Response, AppError> {
    let rows = service()
        .list_cases(&user.tenant_pool, user.company_id, q.stage.as_deref())
        .await?;
    Ok(ApiResponse::success(rows, "Cases fetched"))
}

async fn get_case(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let row = service()
        .get_case(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(row, "Case fetched"))
}

async fn update_case(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInvestmentCase>,
) -> Result<Response, AppError> {
    let row = service()
        .update_case(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::success(row, "Case updated"))
}

async fn advance_stage(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AdvanceStageRequest>,
) -> Result<Response, AppError> {
    let row = service()
        .advance_stage(
            &user.tenant_pool,
            user.company_id,
            id,
            user.user_id,
            &payload,
        )
        .await?;
    Ok(ApiResponse::success(row, "Stage advanced"))
}

async fn create_scenario(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateInvestmentScenario>,
) -> Result<Response, AppError> {
    let row = service()
        .create_scenario(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Scenario created"))
}

async fn list_scenarios(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let _ = service()
        .get_case(&user.tenant_pool, user.company_id, id)
        .await?;
    let rows = service()
        .list_scenarios(&user.tenant_pool, id)
        .await?;
    Ok(ApiResponse::success(rows, "Scenarios fetched"))
}

async fn add_cashflow(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(scenario_id): Path<Uuid>,
    Json(payload): Json<CreateCashflowLine>,
) -> Result<Response, AppError> {
    let row = service()
        .add_cashflow(&user.tenant_pool, scenario_id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Cash flow line added"))
}

async fn list_cashflows(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(scenario_id): Path<Uuid>,
) -> Result<Response, AppError> {
    let rows = service()
        .list_cashflows(&user.tenant_pool, scenario_id)
        .await?;
    Ok(ApiResponse::success(rows, "Cash flows fetched"))
}

async fn recalculate(
    Extension(user): Extension<AuthenticatedTenant>,
    Path((case_id, scenario_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<RecalculateMetricsRequest>,
) -> Result<Response, AppError> {
    let row = service()
        .recalculate_metrics(
            &user.tenant_pool,
            user.company_id,
            case_id,
            scenario_id,
            user.user_id,
            &payload,
        )
        .await?;
    Ok(ApiResponse::success(row, "Metrics recalculated"))
}

async fn create_risk(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateInvestmentRisk>,
) -> Result<Response, AppError> {
    let row = service()
        .create_risk(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::created(row, "Risk added"))
}

async fn list_risks(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let rows = service().list_risks(&user.tenant_pool, id).await?;
    Ok(ApiResponse::success(rows, "Risks fetched"))
}

async fn post_audit(
    Extension(user): Extension<AuthenticatedTenant>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreatePostAudit>,
) -> Result<Response, AppError> {
    let row = service()
        .post_audit(
            &user.tenant_pool,
            user.company_id,
            id,
            user.user_id,
            &payload,
        )
        .await?;
    Ok(ApiResponse::created(row, "Post-audit recorded"))
}

async fn portfolio_board(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let rows = service()
        .portfolio_board(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(rows, "Portfolio board"))
}