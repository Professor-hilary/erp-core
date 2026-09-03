// src/features/capital/handler.rs
use axum::{
    Extension, Router,
    extract::{Json, Path, Query},
    response::Response,
    routing::{get, post},
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    features::capital::{repository::PostgresCapitalRepo, services::CapitalService},
    interface::api::{errors::AppError, responses::ApiResponse},
    middleware::auth::AuthenticatedTenant,
    models::capital::*,
    state::AppState,
};

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // ----- Instruments & Parties -----
        .route(
            "/instruments",
            post(create_instrument).get(list_instruments),
        )
        .route(
            "/instruments/{id}",
            get(get_instrument).put(update_instrument),
        )
        .route(
            "/instruments/{id}/events",
            get(list_events_for_instrument).post(create_event),
        )
        .route("/party", post(create_party).get(list_parties))
        .route("/party/{id}", get(get_party).put(update_party))
        // ----- Facilities -----
        .route("/facilities", post(create_facility).get(list_facilities))
        .route("/facilities/{id}", get(get_facility).put(update_facility))
        .route(
            "/facilities/{id}/lenders",
            post(add_facility_lender).get(list_facility_lenders),
        )
        .route(
            "/facilities/{id}/drawdowns",
            post(create_drawdown).get(list_drawdowns),
        )
        .route(
            "/facilities/{id}/repayments",
            post(create_repayment).get(list_repayments),
        )
        .route(
            "/facilities/{id}/schedules",
            post(create_repayment_schedule).get(list_repayment_schedules),
        )
        .route(
            "/facilities/{id}/accruals",
            post(create_interest_accrual).get(list_interest_accruals),
        )
        .route("/facilities/{id}/fees", post(create_debt_fee))
        .route(
            "/facilities/{id}/covenants",
            post(create_covenant).get(list_covenants),
        )
        .route(
            "/facilities/{id}/covenants-test",
            post(create_covenant_test).get(list_covenants),
        )
        .route("/facilities/{id}/collateral", post(create_collateral))
        .route("/refinancings", post(create_refinancing))
        // ----- Equity -----
        .route(
            "/share-classes",
            post(create_share_class).get(list_share_classes),
        )
        .route("/share-classes/{id}", get(get_share_class))
        .route(
            "/shareholders",
            post(create_shareholder).get(list_shareholders),
        )
        .route("/shareholders/{id}", get(get_shareholding))
        .route(
            "/share-transactions",
            post(create_share_transaction).get(list_share_transactions),
        )
        .route("/dividends", post(create_dividend).get(list_dividends))
        .route("/dividends/{id}/pay", post(pay_dividend))
        .route("/dividends/{id}/payments", get(list_dividend_payments))
        // ----- Equity accounts / retained earnings -----
        .route(
            "/equity-accounts",
            post(create_equity_account).get(list_equity_accounts),
        )
        .route(
            "/equity-movements",
            post(create_equity_movement).get(list_equity_movements),
        )
        // ----- Structure & allocation -----
        .route("/structure-snapshots", post(create_structure_snapshot))
        .route("/structure/latest", get(get_latest_structure))
        .route(
            "/allocations",
            post(create_allocation).get(list_allocations),
        )
        // ----- Projects -----
        .route("/projects", post(create_project).get(list_projects))
        .route("/projects/{id}", get(get_project).put(update_project))
        .route("/projects/{id}/funding", post(add_project_funding))
        // ----- Liquidity -----
        .route("/cash-forecasts", post(create_cash_forecast))
        .route("/cash-forecasts/{id}", get(get_cash_forecast))
        .route(
            "/cash-forecasts/{id}/lines",
            post(add_forecast_line).get(list_forecast_lines),
        )
        // ----- Analytics -----
        .route("/metrics", post(record_metric).get(list_metrics))
        .route("/wacc", post(upsert_wacc))
}

// ---------------------------------------------------------------------------
// Helper – avoids repeating repo + service construction
// ---------------------------------------------------------------------------

fn service() -> CapitalService<PostgresCapitalRepo> {
    CapitalService::new(PostgresCapitalRepo::new())
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListInstrumentsQuery {
    pub family: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFacilitiesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListShareTxQuery {
    pub share_class_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListDividendsQuery {
    pub share_class_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListEquityMovementsQuery {
    pub equity_account_id: Option<Uuid>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsQuery {
    pub metric_code: Option<String>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct AddLenderPayload {
    pub lender_id: Uuid,
    pub commitment_amount: BigDecimal,
    pub participation_pct: Option<BigDecimal>,
    pub is_agent: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct RecordCovenantTestPayload {
    pub test_date: chrono::NaiveDate,
    pub actual_value: Option<BigDecimal>,
    pub is_compliant: Option<bool>,
    pub headroom: Option<BigDecimal>,
    pub notes: Option<String>,
}

// ===========================================================================
// Parties
// ===========================================================================

async fn create_party(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateParty>,
) -> Result<Response, AppError> {
    let instrument = service()
        .create_party(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(instrument, "Party created"))
}

async fn get_party(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let party = service()
        .get_party(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(party, "Party fetched"))
}

async fn list_parties(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_parties(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(items, "Party list fetched"))
}

async fn update_party(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateParty>,
) -> Result<Response, AppError> {
    let party = service()
        .update_party(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::success(party, "Party updated"))
}

// ===========================================================================
// Instruments
// ===========================================================================

async fn create_instrument(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalInstrument>,
) -> Result<Response, AppError> {
    let instrument = service()
        .create_instrument(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(
        instrument,
        "Capital instrument created",
    ))
}

async fn get_instrument(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let instrument = service()
        .get_instrument(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(
        instrument,
        "Capital instrument fetched",
    ))
}

async fn list_instruments(
    Query(q): Query<ListInstrumentsQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_instruments(
            &user.tenant_pool,
            user.company_id,
            q.family.as_deref(),
            q.status.as_deref(),
        )
        .await?;
    Ok(ApiResponse::success(items, "Capital instruments fetched"))
}

async fn update_instrument(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateCapitalInstrument>,
) -> Result<Response, AppError> {
    let instrument = service()
        .update_instrument(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::success(
        instrument,
        "Capital instrument updated",
    ))
}

async fn create_event(
    Path(instrument_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateCapitalEvent>,
) -> Result<Response, AppError> {
    // Ensure the path instrument is used if not supplied in body
    if payload.instrument_id.is_none() {
        payload.instrument_id = Some(instrument_id);
    }
    let event = service()
        .create_event(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(event, "Capital event recorded"))
}

async fn list_events_for_instrument(
    Path(instrument_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let events = service()
        .list_events_for_instrument(&user.tenant_pool, user.company_id, instrument_id)
        .await?;
    Ok(ApiResponse::success(events, "Capital events fetched"))
}

// ===========================================================================
// Facilities
// ===========================================================================

async fn create_facility(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalFacility>,
) -> Result<Response, AppError> {
    let facility = service()
        .create_facility(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(facility, "Capital facility created"))
}

async fn get_facility(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let facility = service()
        .get_facility(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(facility, "Capital facility fetched"))
}

async fn list_facilities(
    Query(q): Query<ListFacilitiesQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let facilities = service()
        .list_facilities(&user.tenant_pool, user.company_id, q.status.as_deref())
        .await?;
    Ok(ApiResponse::success(
        facilities,
        "Capital facilities fetched",
    ))
}

async fn update_facility(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateCapitalFacility>,
) -> Result<Response, AppError> {
    let facility = service()
        .update_facility(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::success(facility, "Capital facility updated"))
}

async fn add_facility_lender(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<AddLenderPayload>,
) -> Result<Response, AppError> {
    let lender = service()
        .add_facility_lender(
            &user.tenant_pool,
            facility_id,
            payload.lender_id,
            payload.commitment_amount,
            payload.participation_pct,
            payload.is_agent.unwrap_or(false),
        )
        .await?;
    Ok(ApiResponse::created(lender, "Facility lender added"))
}

async fn list_facility_lenders(
    Path(facility_id): Path<Uuid>,
    Extension(_user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let lenders = service()
        .list_facility_lenders(&_user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(lenders, "Facility lenders fetched"))
}

// ===========================================================================
// Drawdowns / Repayments / Schedules / Accruals / Fees / Covenants / Collateral
// ===========================================================================

async fn create_drawdown(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateDrawdown>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let drawdown = service()
        .create_drawdown(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(drawdown, "Debt drawdown created"))
}

async fn list_drawdowns(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let drawdowns = service()
        .list_drawdowns(&user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(drawdowns, "Debt drawdowns fetched"))
}

async fn create_repayment(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateRepayment>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let repayment = service()
        .create_repayment(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(repayment, "Debt repayment created"))
}

async fn list_repayments(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let repayments = service()
        .list_repayments(&user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(repayments, "Debt repayments fetched"))
}

async fn create_repayment_schedule(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateRepaymentSchedule>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let schedule = service()
        .create_repayment_schedule(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(schedule, "Repayment schedule created"))
}

async fn list_repayment_schedules(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let schedules = service()
        .list_repayment_schedules(&user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(
        schedules,
        "Repayment schedules fetched",
    ))
}

async fn create_interest_accrual(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateInterestAccrual>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let accrual = service()
        .create_interest_accrual(&user.tenant_pool, &payload, user.user_id)
        .await?;
    Ok(ApiResponse::created(accrual, "Interest accrual recorded"))
}

async fn list_interest_accruals(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let accruals = service()
        .list_interest_accruals(&user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(accruals, "Interest accruals fetched"))
}

async fn create_debt_fee(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateDebtFee>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let fee = service()
        .create_debt_fee(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(fee, "Debt fee recorded"))
}

async fn create_covenant(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateDebtCovenant>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let covenant = service()
        .create_covenant(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(covenant, "Covenant created"))
}

async fn create_covenant_test(
    Path(_facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<DebtCovenantTest>,
) -> Result<Response, AppError> {
    // payload.facility_id = facility_id;
    let covenant = service()
        .record_covenant_test(
            &user.tenant_pool,
            payload.covenant_id,
            payload.test_date,
            payload.actual_value,
            payload.is_compliant,
            payload.headroom,
            payload.notes,
            payload.tested_by,
        )
        .await?;
    Ok(ApiResponse::created(covenant, "Covenant test created"))
}

async fn list_covenants(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let covenants = service()
        .list_covenants(&user.tenant_pool, facility_id)
        .await?;
    Ok(ApiResponse::success(covenants, "Covenants fetched"))
}

async fn create_collateral(
    Path(facility_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateDebtCollateral>,
) -> Result<Response, AppError> {
    payload.facility_id = facility_id;
    let collateral = service()
        .create_collateral(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(collateral, "Collateral recorded"))
}

async fn create_refinancing(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDebtRefinancing>,
) -> Result<Response, AppError> {
    let refinancing = service()
        .create_refinancing(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(refinancing, "Refinancing recorded"))
}

// ===========================================================================
// Equity
// ===========================================================================

async fn create_share_class(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateShareClass>,
) -> Result<Response, AppError> {
    let sc = service()
        .create_share_class(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(sc, "Share class created"))
}

async fn get_share_class(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let sc = service()
        .get_share_class(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(sc, "Share class fetched"))
}

async fn list_share_classes(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_share_classes(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(items, "Share classes fetched"))
}

async fn create_shareholder(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateShareholder>,
) -> Result<Response, AppError> {
    let sh = service()
        .create_shareholder(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(sh, "Shareholder created"))
}

async fn list_shareholders(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_shareholders(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(items, "Shareholders fetched"))
}

async fn get_shareholding(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<Shareholding>,
) -> Result<Response, AppError> {
    let items = service()
        .get_shareholding(
            &user.tenant_pool,
            payload.share_class_id,
            payload.shareholder_id,
        )
        .await?;
    Ok(ApiResponse::success(items, "Shareholding fetched"))
}

async fn create_share_transaction(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateShareTransaction>,
) -> Result<Response, AppError> {
    let tx = service()
        .create_share_transaction(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(tx, "Share transaction recorded"))
}

async fn list_share_transactions(
    Query(q): Query<ListShareTxQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_share_transactions(&user.tenant_pool, user.company_id, q.share_class_id)
        .await?;
    Ok(ApiResponse::success(items, "Share transactions fetched"))
}

async fn create_dividend(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateDividend>,
) -> Result<Response, AppError> {
    let div = service()
        .create_dividend(&user.tenant_pool, user.company_id, &payload, user.user_id)
        .await?;
    Ok(ApiResponse::created(div, "Dividend created"))
}

async fn list_dividends(
    Query(q): Query<ListDividendsQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_dividends(&user.tenant_pool, user.company_id, q.share_class_id)
        .await?;
    Ok(ApiResponse::success(items, "Dividends fetched"))
}

async fn pay_dividend(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<PayDividendRequest>,
) -> Result<Response, AppError> {
    // Resolve GL accounts from company defaults if you have them
    let (payable, cash) = (None, None); // or load from config

    let (dividend, payments) = service()
        .pay_dividend(
            &user.tenant_pool,
            user.company_id,
            user.user_id,
            &payload,
            payable,
            cash,
        )
        .await?;

    Ok(ApiResponse::success(
        json!({ "dividend": dividend, "payments": payments }),
        "Dividend paid",
    ))
}

async fn list_dividend_payments(
    Path(dividend_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let payments = service()
        .list_dividend_payments(&user.tenant_pool, dividend_id)
        .await?;
    Ok(ApiResponse::success(payments, "Dividend payments fetched"))
}
// ===========================================================================
// Equity accounts / movements
// ===========================================================================

async fn create_equity_account(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEquityAccount>,
) -> Result<Response, AppError> {
    let account = service()
        .create_equity_account(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(account, "Equity account created"))
}

async fn list_equity_accounts(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_equity_accounts(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(items, "Equity accounts fetched"))
}

async fn create_equity_movement(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateEquityMovement>,
) -> Result<Response, AppError> {
    let movement = service()
        .create_equity_movement(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(movement, "Equity movement recorded"))
}

async fn list_equity_movements(
    Query(q): Query<ListEquityMovementsQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_equity_movements(
            &user.tenant_pool,
            user.company_id,
            q.equity_account_id,
            q.from,
            q.to,
        )
        .await?;
    Ok(ApiResponse::success(items, "Equity movements fetched"))
}

// ===========================================================================
// Structure & allocations
// ===========================================================================

async fn create_structure_snapshot(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalStructureSnapshot>,
) -> Result<Response, AppError> {
    let snap = service()
        .create_structure_snapshot(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(
        snap,
        "Capital structure snapshot created",
    ))
}

async fn get_latest_structure(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let snap = service()
        .get_latest_structure(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(
        snap,
        "Latest capital structure fetched",
    ))
}

async fn create_allocation(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalAllocation>,
) -> Result<Response, AppError> {
    let allocation = service()
        .create_allocation(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(
        allocation,
        "Capital allocation recorded",
    ))
}

async fn list_allocations(
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_allocations(&user.tenant_pool, user.company_id)
        .await?;
    Ok(ApiResponse::success(items, "Capital allocations fetched"))
}

// ===========================================================================
// Projects
// ===========================================================================

async fn create_project(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalProject>,
) -> Result<Response, AppError> {
    let project = service()
        .create_project(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(project, "Capital project created"))
}

async fn get_project(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let project = service()
        .get_project(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(project, "Capital project fetched"))
}

async fn list_projects(
    Query(q): Query<ListProjectsQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_projects(&user.tenant_pool, user.company_id, q.status.as_deref())
        .await?;
    Ok(ApiResponse::success(items, "Capital projects fetched"))
}

async fn update_project(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<UpdateCapitalProject>,
) -> Result<Response, AppError> {
    let project = service()
        .update_project(&user.tenant_pool, user.company_id, id, &payload)
        .await?;
    Ok(ApiResponse::success(project, "Capital project updated"))
}

async fn add_project_funding(
    Path(project_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(mut payload): Json<CreateProjectFunding>,
) -> Result<Response, AppError> {
    payload.project_id = project_id;
    let funding = service()
        .add_project_funding(&user.tenant_pool, &payload)
        .await?;
    Ok(ApiResponse::created(funding, "Project funding recorded"))
}

// ===========================================================================
// Liquidity
// ===========================================================================

async fn create_cash_forecast(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCashForecast>,
) -> Result<Response, AppError> {
    let forecast = service()
        .create_cash_forecast(&user.tenant_pool, user.company_id, user.user_id, &payload)
        .await?;
    Ok(ApiResponse::created(forecast, "Cash forecast created"))
}

async fn get_cash_forecast(
    Path(id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let forecast = service()
        .get_cash_forecast(&user.tenant_pool, user.company_id, id)
        .await?;
    Ok(ApiResponse::success(forecast, "Cash forecast fetched"))
}

async fn add_forecast_line(
    Path(forecast_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCashForecastLine>,
) -> Result<Response, AppError> {
    let line = service()
        .add_forecast_line(&user.tenant_pool, forecast_id, &payload)
        .await?;
    Ok(ApiResponse::created(line, "Forecast line added"))
}

async fn list_forecast_lines(
    Path(forecast_id): Path<Uuid>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let lines = service()
        .list_forecast_lines(&user.tenant_pool, forecast_id)
        .await?;
    Ok(ApiResponse::success(lines, "Forecast lines fetched"))
}

// ===========================================================================
// Analytics
// ===========================================================================

async fn record_metric(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateCapitalMetric>,
) -> Result<Response, AppError> {
    let metric = service()
        .record_metric(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::created(metric, "Capital metric recorded"))
}

async fn list_metrics(
    Query(q): Query<ListMetricsQuery>,
    Extension(user): Extension<AuthenticatedTenant>,
) -> Result<Response, AppError> {
    let items = service()
        .list_metrics(
            &user.tenant_pool,
            user.company_id,
            q.metric_code.as_deref(),
            q.from,
            q.to,
        )
        .await?;
    Ok(ApiResponse::success(items, "Capital metrics fetched"))
}

async fn upsert_wacc(
    Extension(user): Extension<AuthenticatedTenant>,
    Json(payload): Json<CreateWaccComponent>,
) -> Result<Response, AppError> {
    let wacc = service()
        .upsert_wacc(&user.tenant_pool, user.company_id, &payload)
        .await?;
    Ok(ApiResponse::success(wacc, "WACC component upserted"))
}
