// src/features/accounts/handlers.rs
use crate::{
    features::{
        manufacturing::{repository::ManufacturingRepo, services::ManufacturingService},
        rbac::{Actor, Module},
    },
    interface::api::{errors::AppError, json_errors::AppJson, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::manufacturing::*,
    state::AppState,
};
use axum::{
    Extension, Router,
    extract::{Path, State},
    response::Response,
    routing::{put, /* delete, patch, */ get, post},
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct CreateBomPayload {
    pub header: CreateBomHeaderDto,
    pub lines: Vec<CreateBomLineDto>,
}

/// Pass through client/domain errors (400/403/404/422); only log true internals.
fn map_service_err(e: AppError) -> AppError {
    match &e {
        AppError::Internal(msg) => {
            tracing::error!("Internal error: {msg}");
        }
        AppError::Database(err) => {
            tracing::error!("Database error: {err}");
        }
        _ => {}
    }
    e
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/production-order", post(new_order_route))
        .route("/costing-policy", get(get_costing_policy_route))
        .route("/costing-policy", put(update_costing_policy_route))
        .route("/create/routing/workcenter", post(new_work_center))
        .route("/create/routing/header", post(new_routing))
        .route("/create/routing/operation", post(new_routing_operation))
        .route("/create/overhead-rate", post(new_overhead_rate_route))
        .route("/variance-allocation", post(prorate_variance_route))
        .route(
            "/apply/direct/material",
            post(bulk_raw_material_issuance_route),
        )
        .route(
            "/apply/direct/material/single",
            post(raw_material_issuance_route),
        )
        .route("/apply/direct/labor", post(http_direct_labor_route))
        .route(
            "/labour-allocation/create",
            post(create_labour_allocation_route),
        )
        .route(
            "/labour-allocation/post",
            post(post_labour_allocation_route),
        )
        .route("/labour/categories", get(list_labour_categories_route))
        .route("/labour/categories", post(create_labour_category_route))
        .route("/labour/rates", post(create_labour_rate_route))
        .route("/timesheet/create", post(create_timesheet_route))
        .route("/timesheet/submit", post(submit_timesheet_route))
        .route("/timesheet/approve", post(approve_timesheet_route))
        .route("/timesheet/reject", post(reject_timesheet_route))
        .route("/timesheet/post", post(post_timesheet_route))
        .route("/timesheet/from-operation", post(draft_from_op_route))
        .route("/timesheet/list/{order_uuid}", get(list_timesheets_route))
        .route("/labour/summary/{order_uuid}", get(labour_summary_route))
        .route(
            "/labour/variances/{order_uuid}",
            post(compute_labour_variances_route),
        )
        .route(
            "/labour/report/productivity",
            get(labour_productivity_route),
        )
        .route(
            "/labour/report/productivity/{order_uuid}",
            get(labour_productivity_one_route),
        )
        .route("/labour/report/by-employee", get(labour_by_employee_route))
        .route("/labour/report/by-work-center", get(labour_by_wc_route))
        .route(
            "/labour/report/operation-efficiency",
            get(op_efficiency_route),
        )
        .route(
            "/labour/report/operation-efficiency/{order_uuid}",
            get(op_efficiency_one_route),
        )
        .route("/attendance/upsert", post(upsert_attendance_route))
        .route("/labour/reconcile", get(labour_reconcile_route))
        .route(
            "/labour/reconcile/period",
            get(labour_reconcile_period_route),
        )
        .route("/labour/allocate-payslip", post(allocate_payslip_route))
        .route("/timesheet/team", post(team_timesheet_route))
        .route("/timesheet/bulk-submit", post(bulk_submit_route))
        .route("/timesheet/bulk-approve", post(bulk_approve_route))
        .route("/timesheet/bulk-post", post(bulk_post_route))
        .route("/labour/allocate-payrun", post(allocate_payrun_route))
        .route("/timesheet/from-order", post(draft_from_order_route))
        // .route("/apply/indirect/labor", post(http_indirect_labor_route))
        .route("/apply/overhead/actual", post(actual_overhead_route))
        .route("/apply/overhead/applied", post(applied_overhead_route))
        .route("/start-production", post(start_order_route))
        .route("/operation/ensure", post(ensure_operations_route))
        .route("/operation/list/{order_uuid}", get(list_operations_route))
        .route("/operation/start", post(start_operation_route))
        .route("/operation/complete", post(complete_operation_route))
        .route("/scrap-rework", post(scrap_rework_route))
        .route("/quality-inspection", post(quality_inspection_route))
        .route("/complete-production", post(complete_order_route))
        .route("/create/bom", post(create_bom_route))
        .route("/get/bom/{uuid}", get(get_bom_route))
        .route("/default-product/{uuid}", get(default_bom_route))
}

async fn ensure_operations_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<EnsureOperationsDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.ensure_operation_executions(payload).await {
        Ok(n) => Ok(ApiResponse::success(n, "Operation executions ensured")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn list_operations_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.list_operation_executions(order_uuid).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Operation executions")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn start_operation_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<StartOperationDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.start_operation(payload).await {
        Ok(row) => Ok(ApiResponse::success(row, "Operation started")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn complete_operation_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CompleteOperationDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.complete_operation(payload).await {
        Ok(row) => Ok(ApiResponse::success(row, "Operation completed")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn scrap_rework_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RecordScrapReworkDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.record_scrap_rework(payload, user.user_id).await {
        Ok(row) => Ok(ApiResponse::created(row, "Scrap/rework recorded")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn quality_inspection_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RecordQualityInspectionDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .record_quality_inspection(payload, user.user_id)
        .await
    {
        Ok(row) => Ok(ApiResponse::created(row, "Quality inspection recorded")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/create/bom
/// 1st. STEP: This is the first step, create Bill Oof Materials for production  from Raw Materials available
/// manufacturing.boms and manufacturing.bom_lines are updated. Example: `PRODUCT = CHAIR`
/// > 1. Wood plank -> 5 units
/// > 2. Nails -> 20 units
/// > 3. Vanish -> 1 unit
/// > 4. Cloth -> 1 unit
async fn create_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateBomPayload>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service
        .create_bom(payload.header, payload.lines, user.user_id)
        .await
    {
        Ok(result) => Ok(ApiResponse::created(result, "BOM Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/create/production-order
/// 2nd. STEP: Create a production order, define order no, product id, qty, start and expectdd end date
/// status (Use [Plammed], Completed, in Process, Cancelled), general ledger account
async fn new_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<CreateProductionOrderDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.create_order(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Order Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/create/routing/workcenter
async fn new_work_center(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<WorkCenterDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_work_center(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Work Center Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/create/routing/header
async fn new_routing(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RoutingsDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_routing(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Routing Header Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/create/routing/operation
async fn new_routing_operation(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RoutingOperationsDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.new_routing_operation(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Routing Operation Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/materials-issuance
/// 3rd Prepare Materials for production, Debit WIP Credit Raw Materials
async fn bulk_raw_material_issuance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<IssueMaterialDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.issue")?;
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.bulk_issue_material(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Material Issuance Successful")),
        Err(e) => Err(map_service_err(e)),
    }
}

///# POST /manufacturing/materials-issuance
/// 3rd Prepare Materials for production, Debit WIP Credit Raw Materials
async fn raw_material_issuance_route(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<SingleMaterialIssueDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.issue")?;
    // Get the resolver for this tenant.
    // Clone the Arc so we don't hold the DashMap guard across an await.
    let resolver = state
        .account_role_maps
        .get(&user.company_id)
        .map(|entry| Arc::clone(entry.value()))
        .ok_or_else(|| {
            AppError::Internal("Account-role resolver is not loaded for this company".into())
        })?;

    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    /*match service.issue_material(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Material Issuance Successful")),
        Err(e) => Err(map_service_err(e)),
    }*/
    let result = service
        .issue_material(&resolver, payload, user.user_id)
        .await?;

    Ok(ApiResponse::created(result, "Material Issuance Successful"))
}

///# POST /manufacturing/create/overhead-rate
async fn new_overhead_rate_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateOverheadRateDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.create_overhead_rate(payload).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Order Created")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/variance-allocation
async fn prorate_variance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ProrateVarianceDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.prorate_variance(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Proration Successful")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/overhead-application
/// This is invoiced and paid overhead by the accountant or resource disbursement
/// which may differ from rated overhead, typically ran when money is available
async fn actual_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RecognizeOverhead>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.recognize_overhead_paid(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Actual Overhead Recognized")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/overhead-application
/// Used with overhead rates to cost production, such as, based on machine hour,
/// labor, material cost, etc. Assumed Overhead is approximated rather than actual.
async fn applied_overhead_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<ApplyOverheadDto>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.apply_rated_overhead(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Applied Overhead Applied")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/labour-allocation/create

async fn list_labour_categories_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.list_labour_categories().await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Labour categories")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn create_labour_category_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateLabourCategoryDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.create_labour_category(payload).await {
        Ok(row) => Ok(ApiResponse::created(row, "Labour category created")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn create_labour_rate_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateLabourRateDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.create_labour_rate(payload).await {
        Ok(row) => Ok(ApiResponse::created(row, "Labour rate created")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn create_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<CreateLabourTimeEntryDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.timesheet")?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .create_labour_time_entry(payload, user.user_id)
        .await
    {
        Ok(row) => Ok(ApiResponse::created(row, "Timesheet draft created")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn submit_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<serde_json::Value>,
) -> Result<Response, AppError> {
    let entry_uuid: Uuid = payload
        .get("entry_uuid")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::BadRequest("entry_uuid required".into()))?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.submit_labour_time_entry(entry_uuid).await {
        Ok(row) => Ok(ApiResponse::success(row, "Timesheet submitted")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn approve_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<ApproveLabourTimeEntryDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.timesheet")?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .approve_labour_time_entry(payload, user.user_id)
        .await
    {
        Ok(row) => Ok(ApiResponse::success(row, "Timesheet approved")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn reject_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<RejectLabourTimeEntryDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.reject_labour_time_entry(payload).await {
        Ok(row) => Ok(ApiResponse::success(row, "Timesheet rejected")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn post_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<PostLabourTimeEntryDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.timesheet")?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.post_labour_time_entry(payload, user.user_id).await {
        Ok(row) => Ok(ApiResponse::created(row, "Actual labour posted")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn draft_from_op_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<DraftTimeFromOperationDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .draft_time_from_operation(payload, user.user_id)
        .await
    {
        Ok(row) => Ok(ApiResponse::created(
            row,
            "Timesheet drafted from operation",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn list_timesheets_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.list_labour_time_entries_for_order(order_uuid).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Timesheets")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn team_timesheet_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateTeamTimeEntriesDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .create_team_time_entries(payload, user.user_id)
        .await
    {
        Ok(rows) => Ok(ApiResponse::created(rows, "Team timesheets created")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn bulk_submit_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<BulkTimeEntryIdsDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.bulk_submit_time_entries(payload.entry_uuids).await {
        Ok(n) => Ok(ApiResponse::success(n, "Timesheets submitted")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn bulk_approve_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<BulkTimeEntryIdsDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .bulk_approve_time_entries(payload, user.user_id)
        .await
    {
        Ok(n) => Ok(ApiResponse::success(n, "Timesheets approved")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn bulk_post_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<BulkTimeEntryIdsDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.bulk_post_time_entries(payload, user.user_id).await {
        Ok(n) => Ok(ApiResponse::success(n, "Timesheets posted to WIP")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn allocate_payrun_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<AllocatePayrunLabourDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.allocate_payrun_labour(payload, user.user_id).await {
        Ok(rows) => Ok(ApiResponse::created(
            rows,
            "Payrun labour allocation results",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn draft_from_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<DraftTimeFromOrderDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .draft_time_from_order_ops(payload.production_order_uuid, user.user_id)
        .await
    {
        Ok(rows) => Ok(ApiResponse::created(
            rows,
            "Timesheets drafted from operations",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn upsert_attendance_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<UpsertAttendanceDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.upsert_attendance(payload, user.user_id).await {
        Ok(row) => Ok(ApiResponse::created(row, "Attendance saved")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_reconcile_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_hours_reconciliation(None).await {
        Ok(rows) => Ok(ApiResponse::success(
            rows,
            "Attendance vs timesheet reconciliation",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_reconcile_period_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_hours_reconciliation_period().await {
        Ok(rows) => Ok(ApiResponse::success(
            rows,
            "Monthly labour hours reconciliation",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn allocate_payslip_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<AllocatePayslipLabourDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.allocate_payslip_labour(payload, user.user_id).await {
        Ok(rows) => Ok(ApiResponse::created(
            rows,
            "Payslip labour allocated to orders",
        )),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_productivity_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_productivity_by_order(None).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Labour productivity by order")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_productivity_one_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_productivity_by_order(Some(order_uuid)).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Labour productivity")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_by_employee_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_by_employee().await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Labour by employee")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_by_wc_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.labour_by_work_center().await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Labour by work centre")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn op_efficiency_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.operation_efficiency(None).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Operation efficiency")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn op_efficiency_one_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.operation_efficiency(Some(order_uuid)).await {
        Ok(rows) => Ok(ApiResponse::success(rows, "Operation efficiency")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn compute_labour_variances_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.compute_labour_variances(order_uuid, None).await {
        Ok(row) => Ok(ApiResponse::created(row, "Labour variances computed")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn labour_summary_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    axum::extract::Path(order_uuid): axum::extract::Path<Uuid>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.production_order_labour_summary(order_uuid).await {
        Ok(row) => Ok(ApiResponse::success(row, "Labour cost summary")),
        Err(e) => Err(map_service_err(e)),
    }
}

async fn create_labour_allocation_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<CreateLabourAllocationDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service
        .create_labour_allocation(payload, user.user_id)
        .await
    {
        Ok(result) => Ok(ApiResponse::created(result, "Labour allocation created")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/labour-allocation/post
async fn post_labour_allocation_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    AppJson(payload): AppJson<PostLabourAllocationDto>,
) -> Result<Response, AppError> {
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.post_labour_allocation(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Labour allocation posted")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/apply/direct/labor
async fn http_direct_labor_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<ApplyDirectLaborDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service
        .apply_direct_labor_costs(payload, user.user_id)
        .await
    {
        Ok(result) => Ok(ApiResponse::created(result, "Direct Labor Costs Applied")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/apply/indirect/labor
// async fn http_indirect_labor_route(
//     State(_state): State<Arc<AppState>>,
//     Extension(user): Extension<AuthenticatedTenant>,
//     AppJson(payload): AppJson<ApplyInDirectLaborDto>,
// ) -> Result<Response, AppError> {
//     let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
//     let service: ManufacturingService = ManufacturingService::new(repo);

//     match service.apply_indirect_labor_costs(payload, user.user_id).await {
//         Ok(result) => Ok(ApiResponse::created(result, "Inirect Labor Costs Applied")),
//         Err(e) => Err(map_service_err(e)),
//     }
// }

/// POST /manufacturing/start-production
/// Explicit Planned → In Progress before costs / completion.
async fn start_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<StartProductionOrderDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.start_order(payload).await {
        Ok(result) => Ok(ApiResponse::success(result, "Production started")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// POST /manufacturing/complete-production
async fn complete_order_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<CompleteProductionOrderDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.complete_order(payload, user.user_id).await {
        Ok(result) => Ok(ApiResponse::created(result, "Production Completed")),
        Err(e) => Err(map_service_err(e)),
    }
}

/// GET /manufacturing/bom/:uuid
async fn get_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.get_bom(uuid).await {
        Ok(Some(result)) => Ok(ApiResponse::success(result, "BOM Created")),
        Ok(None) => Err(AppError::NotFound("BOM not found".into())),
        Err(e) => Err(map_service_err(e)),
    }
}

///# GET /manufacturing/default-product/{uuid}
async fn default_bom_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Path(uuid): Path<Uuid>,
) -> Result<Response, AppError> {
    let repo: ManufacturingRepo = ManufacturingRepo::new(user.tenant_pool);
    let service: ManufacturingService = ManufacturingService::new(repo);

    match service.default_bom(uuid).await {
        Ok(Some(result)) => Ok(ApiResponse::success(result, "BOM found")),
        Ok(None) => Err(AppError::NotFound("No default OM found".into())),
        Err(e) => Err(map_service_err(e)),
    }
}


async fn get_costing_policy_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.costing_policy().await {
        Ok(p) => {
            let rules = p.product_cost_rules();
            Ok(ApiResponse::success(
                CostingPolicyResponse { policy: p, rules },
                "Costing policy",
            ))
        }
        Err(e) => Err(map_service_err(e)),
    }
}

async fn update_costing_policy_route(
    State(_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedTenant>,
    Extension(actor): Extension<Actor>,
    AppJson(payload): AppJson<UpdateCostingPolicyDto>,
) -> Result<Response, AppError> {
    actor.require_module(Module::Manufacturing)?;
    actor.require("manufacturing.manage")?;
    let repo = ManufacturingRepo::new(user.tenant_pool);
    let service = ManufacturingService::new(repo);
    match service.update_costing_policy(payload, user.user_id).await {
        Ok(p) => {
            let rules = p.product_cost_rules();
            Ok(ApiResponse::success(
                CostingPolicyResponse { policy: p, rules },
                "Costing policy updated",
            ))
        }
        Err(e) => Err(map_service_err(e)),
    }
}
