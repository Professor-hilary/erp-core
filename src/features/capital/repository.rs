use crate::{interface::api::errors::AppError, models::capital::*};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder, Transaction};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Repository trait – comprehensive capital domain
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CapitalRepository: Send + Sync {
    // =========================================================================
    // 1. Capital Instruments (unified register)
    // =========================================================================
    async fn create_instrument(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError>;

    async fn get_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalInstrument, AppError>;

    async fn list_instruments(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        family: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<CapitalInstrument>, AppError>;

    async fn update_instrument(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError>;

    // =========================================================================
    // 2. Capital Events (audit / lifecycle)
    // =========================================================================
    async fn create_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalEvent,
    ) -> Result<CapitalEvent, AppError>;

    async fn list_events_for_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        instrument_id: Uuid,
    ) -> Result<Vec<CapitalEvent>, AppError>;

    // =========================================================================
    // 3. Debt Facilities
    // =========================================================================
    async fn create_facility(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalFacility,
    ) -> Result<CapitalFacility, AppError>;

    async fn get_facility(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError>;

    async fn list_facilities(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<CapitalFacility>, AppError>;

    async fn update_facility(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalFacility,
    ) -> Result<CapitalFacility, AppError>;

    async fn add_facility_lender(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        facility_id: Uuid,
        lender_id: Uuid,
        commitment_amount: rust_decimal::Decimal,
        participation_pct: Option<rust_decimal::Decimal>,
        is_agent: bool,
    ) -> Result<FacilityLender, AppError>;

    async fn list_facility_lenders(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<FacilityLender>, AppError>;

    // =========================================================================
    // 4. Drawdowns
    // =========================================================================
    async fn create_drawdown(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateDrawdown,
    ) -> Result<DebtDrawdown, AppError>;

    async fn list_drawdowns(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError>;

    // =========================================================================
    // 5. Repayment Schedules & Repayments
    // =========================================================================
    async fn create_repayment_schedule(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateRepaymentSchedule,
    ) -> Result<DebtRepaymentSchedule, AppError>;

    async fn list_repayment_schedules(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepaymentSchedule>, AppError>;

    async fn create_repayment(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateRepayment,
    ) -> Result<DebtRepayment, AppError>;

    async fn list_repayments(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError>;

    // =========================================================================
    // 6. Interest Accruals & Fees
    // =========================================================================
    async fn create_interest_accrual(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateInterestAccrual,
    ) -> Result<DebtInterestAccrual, AppError>;

    async fn list_interest_accruals(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtInterestAccrual>, AppError>;

    async fn create_debt_fee(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtFee,
    ) -> Result<DebtFee, AppError>;

    // =========================================================================
    // 7. Covenants
    // =========================================================================
    async fn create_covenant(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtCovenant,
    ) -> Result<DebtCovenant, AppError>;

    async fn list_covenants(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtCovenant>, AppError>;

    async fn record_covenant_test(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        covenant_id: Uuid,
        test_date: NaiveDate,
        actual_value: Option<rust_decimal::Decimal>,
        is_compliant: Option<bool>,
        headroom: Option<rust_decimal::Decimal>,
        notes: Option<&str>,
        tested_by: Uuid,
    ) -> Result<DebtCovenantTest, AppError>;

    // =========================================================================
    // 8. Collateral & Refinancing
    // =========================================================================
    async fn create_collateral(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtCollateral,
    ) -> Result<DebtCollateral, AppError>;

    async fn create_refinancing(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateDebtRefinancing,
    ) -> Result<DebtRefinancing, AppError>;

    // =========================================================================
    // 9. Equity – Share Classes & Shareholders
    // =========================================================================
    async fn create_share_class(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateShareClass,
    ) -> Result<ShareClass, AppError>;

    async fn get_share_class(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<ShareClass, AppError>;

    async fn list_share_classes(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<ShareClass>, AppError>;

    async fn create_shareholder(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateShareholder,
    ) -> Result<Shareholder, AppError>;

    async fn list_shareholders(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<Shareholder>, AppError>;

    async fn get_shareholding(
        &self,
        pool: &PgPool,
        share_class_id: Uuid,
        shareholder_id: Uuid,
    ) -> Result<Option<Shareholding>, AppError>;

    // =========================================================================
    // 10. Share Transactions & Dividends
    // =========================================================================
    async fn create_share_transaction(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateShareTransaction,
    ) -> Result<ShareTransaction, AppError>;

    async fn list_share_transactions(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        share_class_id: Option<Uuid>,
    ) -> Result<Vec<ShareTransaction>, AppError>;

    async fn create_dividend(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateDividend,
    ) -> Result<Dividend, AppError>;

    async fn list_dividends(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        share_class_id: Option<Uuid>,
    ) -> Result<Vec<Dividend>, AppError>;

    // =========================================================================
    // 11. Equity Accounts & Movements (Retained Earnings etc.)
    // =========================================================================
    async fn create_equity_account(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateEquityAccount,
    ) -> Result<EquityAccount, AppError>;

    async fn list_equity_accounts(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<EquityAccount>, AppError>;

    async fn create_equity_movement(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateEquityMovement,
    ) -> Result<EquityMovement, AppError>;

    async fn list_equity_movements(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        equity_account_id: Option<Uuid>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<EquityMovement>, AppError>;

    // =========================================================================
    // 12. Capital Structure & Allocations
    // =========================================================================
    async fn create_structure_snapshot(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateCapitalStructureSnapshot,
    ) -> Result<CapitalStructureSnapshot, AppError>;

    async fn get_latest_structure(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Option<CapitalStructureSnapshot>, AppError>;

    async fn create_allocation(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateCapitalAllocation,
    ) -> Result<CapitalAllocation, AppError>;

    async fn list_allocations(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<CapitalAllocation>, AppError>;

    // =========================================================================
    // 13. Projects & Deployment
    // =========================================================================
    async fn create_project(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateCapitalProject,
    ) -> Result<CapitalProject, AppError>;

    async fn get_project(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalProject, AppError>;

    async fn list_projects(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<CapitalProject>, AppError>;

    async fn update_project(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalProject,
    ) -> Result<CapitalProject, AppError>;

    async fn add_project_funding(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateProjectFunding,
    ) -> Result<ProjectFunding, AppError>;

    // =========================================================================
    // 14. Liquidity / Cash Forecasts
    // =========================================================================
    async fn create_cash_forecast(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCashForecast,
    ) -> Result<CashForecast, AppError>;

    async fn add_forecast_line(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        forecast_id: Uuid,
        payload: &CreateCashForecastLine,
    ) -> Result<CashForecastLine, AppError>;

    async fn get_cash_forecast(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CashForecast, AppError>;

    async fn list_forecast_lines(
        &self,
        pool: &PgPool,
        forecast_id: Uuid,
    ) -> Result<Vec<CashForecastLine>, AppError>;

    // =========================================================================
    // 15. Analytics
    // =========================================================================
    async fn record_metric(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateCapitalMetric,
    ) -> Result<CapitalMetric, AppError>;

    async fn list_metrics(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        metric_code: Option<&str>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CapitalMetric>, AppError>;

    async fn upsert_wacc(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateWaccComponent,
    ) -> Result<WaccComponent, AppError>;
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

pub struct PostgresCapitalRepo;

impl PostgresCapitalRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CapitalRepository for PostgresCapitalRepo {
    // -------------------------------------------------------------------------
    // Instruments
    // -------------------------------------------------------------------------
    async fn create_instrument(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        _user_id: Uuid,
        payload: &CreateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError> {
        let row = sqlx::query_as::<_, CapitalInstrument>(
            r#"
            INSERT INTO capital.capital_instruments (
                company_id, instrument_code, name, instrument_family, instrument_type,
                currency_id, original_principal, outstanding_principal, face_value,
                issue_price, effective_date, maturity_date, is_perpetual, ranking,
                is_callable, is_putable, is_convertible, conversion_ratio, conversion_price,
                status, accounting_treatment, notes
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $7, $8,
                $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18,
                COALESCE($19, 'active'), $20, $21
            )
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(&payload.instrument_code)
        .bind(&payload.name)
        .bind(&payload.instrument_family)
        .bind(&payload.instrument_type)
        .bind(payload.currency_id)
        .bind(payload.original_principal)
        .bind(payload.face_value)
        .bind(payload.issue_price)
        .bind(payload.effective_date)
        .bind(payload.maturity_date)
        .bind(payload.is_perpetual.unwrap_or(false))
        .bind(&payload.ranking)
        .bind(payload.is_callable.unwrap_or(false))
        .bind(payload.is_putable.unwrap_or(false))
        .bind(payload.is_convertible.unwrap_or(false))
        .bind(payload.conversion_ratio)
        .bind(payload.conversion_price)
        .bind(&payload.status)
        .bind(&payload.accounting_treatment)
        .bind(&payload.notes)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn get_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalInstrument, AppError> {
        sqlx::query_as::<_, CapitalInstrument>(
            r#"
            SELECT * FROM capital.capital_instruments
            WHERE id = $1 AND company_id = $2
            "#,
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital instrument not found".into()))
    }

    async fn list_instruments(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        family: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<CapitalInstrument>, AppError> {
        let mut qb =
            QueryBuilder::new("SELECT * FROM capital.capital_instruments WHERE company_id = ");
        qb.push_bind(company_id);

        if let Some(f) = family {
            qb.push(" AND instrument_family = ");
            qb.push_bind(f);
        }
        if let Some(s) = status {
            qb.push(" AND status = ");
            qb.push_bind(s);
        }
        qb.push(" ORDER BY effective_date DESC, created_at DESC");

        let rows = qb
            .build_query_as::<CapitalInstrument>()
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn update_instrument(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError> {
        let row = sqlx::query_as::<_, CapitalInstrument>(
            r#"
            UPDATE capital.capital_instruments SET
                name                 = COALESCE($3, name),
                outstanding_principal = COALESCE($4, outstanding_principal),
                status               = COALESCE($5, status),
                maturity_date        = COALESCE($6, maturity_date),
                notes                = COALESCE($7, notes),
                updated_at           = now()
            WHERE id = $1 AND company_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&payload.name)
        .bind(payload.outstanding_principal)
        .bind(&payload.status)
        .bind(payload.maturity_date)
        .bind(&payload.notes)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital instrument not found".into()))?;

        Ok(row)
    }

    // -------------------------------------------------------------------------
    // Events
    // -------------------------------------------------------------------------
    async fn create_event(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalEvent,
    ) -> Result<CapitalEvent, AppError> {
        let row = sqlx::query_as::<_, CapitalEvent>(
            r#"
            INSERT INTO capital.capital_events (
                company_id, instrument_id, event_type, event_date, effective_date,
                amount, currency_id, shares, description, related_party_id,
                journal_entry_id, created_by
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(payload.instrument_id)
        .bind(&payload.event_type)
        .bind(payload.event_date)
        .bind(payload.effective_date)
        .bind(payload.amount)
        .bind(payload.currency_id)
        .bind(payload.shares)
        .bind(&payload.description)
        .bind(payload.related_party_id)
        .bind(payload.journal_entry_id)
        .bind(user_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_events_for_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        instrument_id: Uuid,
    ) -> Result<Vec<CapitalEvent>, AppError> {
        let rows = sqlx::query_as::<_, CapitalEvent>(
            r#"
            SELECT * FROM capital.capital_events
            WHERE company_id = $1 AND instrument_id = $2
            ORDER BY event_date DESC, created_at DESC
            "#,
        )
        .bind(company_id)
        .bind(instrument_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    // -------------------------------------------------------------------------
    // Facilities
    // -------------------------------------------------------------------------
    async fn create_facility(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        _user_id: Uuid,
        payload: &CreateCapitalFacility,
    ) -> Result<CapitalFacility, AppError> {
        let facility = sqlx::query_as::<_, CapitalFacility>(
            r#"
            INSERT INTO capital.debt_facilities (
                company_id, instrument_id, facility_code, facility_name, facility_type,
                agent_id, currency_id, committed_amount, available_amount, drawn_amount,
                interest_type, base_rate_index, margin_bps, floor_rate, ceiling_rate,
                day_count_convention, payment_frequency, amortization_type,
                effective_date, maturity_date, commitment_fee_bps, utilization_fee_bps,
                prepayment_penalty, collateral_required, is_secured, ranking, status
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $8, 0,
                $9, $10, $11, $12, $13,
                COALESCE($14, 'ACT/360'), $15, $16,
                $17, $18, $19, $20,
                $21, COALESCE($22, false), COALESCE($23, false), $24,
                COALESCE($25, 'active')
            )
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(payload.instrument_id)
        .bind(&payload.facility_code)
        .bind(&payload.facility_name)
        .bind(&payload.facility_type)
        .bind(payload.agent_id)
        .bind(payload.currency_id)
        .bind(payload.committed_amount)
        .bind(&payload.interest_type)
        .bind(&payload.base_rate_index)
        .bind(payload.margin_bps)
        .bind(payload.floor_rate)
        .bind(payload.ceiling_rate)
        .bind(&payload.day_count_convention)
        .bind(&payload.payment_frequency)
        .bind(&payload.amortization_type)
        .bind(payload.effective_date)
        .bind(payload.maturity_date)
        .bind(payload.commitment_fee_bps)
        .bind(payload.utilization_fee_bps)
        .bind(&payload.prepayment_penalty)
        .bind(payload.collateral_required)
        .bind(payload.is_secured)
        .bind(&payload.ranking)
        .bind(&payload.status)
        .fetch_one(&mut **tx)
        .await?;

        Ok(facility)
    }

    async fn get_facility(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError> {
        sqlx::query_as::<_, CapitalFacility>(
            r#"
            SELECT * FROM capital.debt_facilities
            WHERE id = $1 AND company_id = $2
            "#,
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital facility not found".into()))
    }

    async fn list_facilities(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<CapitalFacility>, AppError> {
        let mut qb = QueryBuilder::new("SELECT * FROM capital.debt_facilities WHERE company_id = ");
        qb.push_bind(company_id);

        if let Some(s) = status {
            qb.push(" AND status = ");
            qb.push_bind(s);
        }
        qb.push(" ORDER BY created_at DESC");

        let rows = qb
            .build_query_as::<CapitalFacility>()
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn update_facility(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalFacility,
    ) -> Result<CapitalFacility, AppError> {
        let row = sqlx::query_as::<_, CapitalFacility>(
            r#"
            UPDATE capital.debt_facilities SET
                facility_name     = COALESCE($3, facility_name),
                available_amount  = COALESCE($4, available_amount),
                drawn_amount      = COALESCE($5, drawn_amount),
                margin_bps        = COALESCE($6, margin_bps),
                maturity_date     = COALESCE($7, maturity_date),
                status            = COALESCE($8, status),
                updated_at        = now()
            WHERE id = $1 AND company_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&payload.facility_name)
        .bind(payload.available_amount)
        .bind(payload.drawn_amount)
        .bind(payload.margin_bps)
        .bind(payload.maturity_date)
        .bind(&payload.status)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital facility not found".into()))?;

        Ok(row)
    }

    async fn add_facility_lender(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        facility_id: Uuid,
        lender_id: Uuid,
        commitment_amount: rust_decimal::Decimal,
        participation_pct: Option<rust_decimal::Decimal>,
        is_agent: bool,
    ) -> Result<FacilityLender, AppError> {
        let row = sqlx::query_as::<_, FacilityLender>(
            r#"
            INSERT INTO capital.debt_facility_lenders (
                facility_id, lender_id, commitment_amount, participation_pct, is_agent
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(facility_id)
        .bind(lender_id)
        .bind(commitment_amount)
        .bind(participation_pct)
        .bind(is_agent)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_facility_lenders(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<FacilityLender>, AppError> {
        let rows = sqlx::query_as::<_, FacilityLender>(
            r#"
            SELECT * FROM capital.debt_facility_lenders
            WHERE facility_id = $1
            ORDER BY is_agent DESC, commitment_amount DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    // -------------------------------------------------------------------------
    // Drawdowns
    // -------------------------------------------------------------------------
    async fn create_drawdown(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _user_id: Uuid,
        payload: &CreateDrawdown,
    ) -> Result<DebtDrawdown, AppError> {
        // Optionally: update facility drawn_amount / available_amount in same tx
        let drawdown = sqlx::query_as::<_, DebtDrawdown>(
            r#"
            INSERT INTO capital.debt_drawdowns (
                facility_id, instrument_id, drawdown_date, value_date,
                amount, currency_id, reference, purpose, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE($9, 'posted'))
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.instrument_id)
        .bind(payload.drawdown_date)
        .bind(payload.value_date)
        .bind(payload.amount)
        .bind(payload.currency_id)
        .bind(&payload.reference)
        .bind(&payload.purpose)
        .bind(&payload.status)
        .fetch_one(&mut **tx)
        .await?;

        // Keep facility balances in sync
        sqlx::query(
            r#"
            UPDATE capital.debt_facilities
            SET drawn_amount     = drawn_amount + $2,
                available_amount = available_amount - $2,
                updated_at       = now()
            WHERE id = $1
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.amount)
        .execute(&mut **tx)
        .await?;

        Ok(drawdown)
    }

    async fn list_drawdowns(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError> {
        let rows = sqlx::query_as::<_, DebtDrawdown>(
            r#"
            SELECT * FROM capital.debt_drawdowns
            WHERE facility_id = $1
            ORDER BY drawdown_date DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    // -------------------------------------------------------------------------
    // Schedules & Repayments
    // -------------------------------------------------------------------------
    async fn create_repayment_schedule(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateRepaymentSchedule,
    ) -> Result<DebtRepaymentSchedule, AppError> {
        let row = sqlx::query_as::<_, DebtRepaymentSchedule>(
            r#"
            INSERT INTO capital.debt_repayment_schedules (
                facility_id, instrument_id, sequence_no, due_date,
                principal_due, interest_due, fee_due, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'pending'))
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.instrument_id)
        .bind(payload.sequence_no)
        .bind(payload.due_date)
        .bind(payload.principal_due)
        .bind(payload.interest_due)
        .bind(payload.fee_due.unwrap_or_default())
        .bind(&payload.status)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_repayment_schedules(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepaymentSchedule>, AppError> {
        let rows = sqlx::query_as::<_, DebtRepaymentSchedule>(
            r#"
            SELECT * FROM capital.debt_repayment_schedules
            WHERE facility_id = $1
            ORDER BY sequence_no
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn create_repayment(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _user_id: Uuid,
        payload: &CreateRepayment,
    ) -> Result<DebtRepayment, AppError> {
        let repayment = sqlx::query_as::<_, DebtRepayment>(
            r#"
            INSERT INTO capital.debt_repayments (
                facility_id, schedule_id, repayment_date,
                principal_amount, interest_amount, fee_amount,
                currency_id, payment_method, reference, journal_entry_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.schedule_id)
        .bind(payload.repayment_date)
        .bind(payload.principal_amount)
        .bind(payload.interest_amount)
        .bind(payload.fee_amount.unwrap_or_default())
        .bind(payload.currency_id)
        .bind(&payload.payment_method)
        .bind(&payload.reference)
        .bind(payload.journal_entry_id)
        .fetch_one(&mut **tx)
        .await?;

        // Reduce drawn principal
        if payload.principal_amount > rust_decimal::Decimal::ZERO {
            sqlx::query(
                r#"
                UPDATE capital.debt_facilities
                SET drawn_amount = GREATEST(drawn_amount - $2, 0),
                    updated_at   = now()
                WHERE id = $1
                "#,
            )
            .bind(payload.facility_id)
            .bind(payload.principal_amount)
            .execute(&mut **tx)
            .await?;
        }

        Ok(repayment)
    }

    async fn list_repayments(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError> {
        let rows = sqlx::query_as::<_, DebtRepayment>(
            r#"
            SELECT * FROM capital.debt_repayments
            WHERE facility_id = $1
            ORDER BY repayment_date DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    // -------------------------------------------------------------------------
    // Interest & Fees
    // -------------------------------------------------------------------------
    async fn create_interest_accrual(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateInterestAccrual,
    ) -> Result<DebtInterestAccrual, AppError> {
        let row = sqlx::query_as::<_, DebtInterestAccrual>(
            r#"
            INSERT INTO capital.debt_interest_accruals (
                facility_id, instrument_id, period_start, period_end,
                principal_base, annual_rate, day_count, interest_amount,
                is_paid, journal_entry_id
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,COALESCE($9,false),$10)
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.instrument_id)
        .bind(payload.period_start)
        .bind(payload.period_end)
        .bind(payload.principal_base)
        .bind(payload.annual_rate)
        .bind(payload.day_count)
        .bind(payload.interest_amount)
        .bind(payload.is_paid)
        .bind(payload.journal_entry_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_interest_accruals(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtInterestAccrual>, AppError> {
        let rows = sqlx::query_as::<_, DebtInterestAccrual>(
            r#"
            SELECT * FROM capital.debt_interest_accruals
            WHERE facility_id = $1
            ORDER BY period_end DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn create_debt_fee(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtFee,
    ) -> Result<DebtFee, AppError> {
        let row = sqlx::query_as::<_, DebtFee>(
            r#"
            INSERT INTO capital.debt_fees (
                facility_id, fee_type, fee_date, amount, currency_id, description, journal_entry_id
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(&payload.fee_type)
        .bind(payload.fee_date)
        .bind(payload.amount)
        .bind(payload.currency_id)
        .bind(&payload.description)
        .bind(payload.journal_entry_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    // -------------------------------------------------------------------------
    // Covenants
    // -------------------------------------------------------------------------
    async fn create_covenant(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtCovenant,
    ) -> Result<DebtCovenant, AppError> {
        let row = sqlx::query_as::<_, DebtCovenant>(
            r#"
            INSERT INTO capital.debt_covenants (
                facility_id, covenant_code, covenant_name, covenant_type,
                metric, operator, threshold_min, threshold_max,
                measurement_frequency, testing_date_rule, effective_date,
                expiry_date, cure_period_days, status
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,COALESCE($14,'active'))
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(&payload.covenant_code)
        .bind(&payload.covenant_name)
        .bind(&payload.covenant_type)
        .bind(&payload.metric)
        .bind(&payload.operator)
        .bind(payload.threshold_min)
        .bind(payload.threshold_max)
        .bind(&payload.measurement_frequency)
        .bind(&payload.testing_date_rule)
        .bind(payload.effective_date)
        .bind(payload.expiry_date)
        .bind(payload.cure_period_days)
        .bind(&payload.status)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_covenants(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtCovenant>, AppError> {
        let rows = sqlx::query_as::<_, DebtCovenant>(
            r#"
            SELECT * FROM capital.debt_covenants
            WHERE facility_id = $1
            ORDER BY effective_date
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn record_covenant_test(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        covenant_id: Uuid,
        test_date: NaiveDate,
        actual_value: Option<rust_decimal::Decimal>,
        is_compliant: Option<bool>,
        headroom: Option<rust_decimal::Decimal>,
        notes: Option<&str>,
        tested_by: Uuid,
    ) -> Result<DebtCovenantTest, AppError> {
        let row = sqlx::query_as::<_, DebtCovenantTest>(
            r#"
            INSERT INTO capital.debt_covenant_tests (
                covenant_id, test_date, actual_value, is_compliant, headroom, notes, tested_by
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
            "#,
        )
        .bind(covenant_id)
        .bind(test_date)
        .bind(actual_value)
        .bind(is_compliant)
        .bind(headroom)
        .bind(notes)
        .bind(tested_by)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    // -------------------------------------------------------------------------
    // Collateral & Refinancing
    // -------------------------------------------------------------------------
    async fn create_collateral(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        payload: &CreateDebtCollateral,
    ) -> Result<DebtCollateral, AppError> {
        let row = sqlx::query_as::<_, DebtCollateral>(
            r#"
            INSERT INTO capital.debt_collateral (
                facility_id, collateral_type, description, estimated_value,
                currency_id, valuation_date, ranking, is_perfected
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,COALESCE($8,false))
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(&payload.collateral_type)
        .bind(&payload.description)
        .bind(payload.estimated_value)
        .bind(payload.currency_id)
        .bind(payload.valuation_date)
        .bind(&payload.ranking)
        .bind(payload.is_perfected)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn create_refinancing(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateDebtRefinancing,
    ) -> Result<DebtRefinancing, AppError> {
        let row = sqlx::query_as::<_, DebtRefinancing>(
            r#"
            INSERT INTO capital.debt_refinancings (
                company_id, old_facility_id, new_facility_id,
                refinancing_date, principal_refinanced, costs, description, journal_entry_id
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(payload.old_facility_id)
        .bind(payload.new_facility_id)
        .bind(payload.refinancing_date)
        .bind(payload.principal_refinanced)
        .bind(payload.costs.unwrap_or_default())
        .bind(&payload.description)
        .bind(payload.journal_entry_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    // -------------------------------------------------------------------------
    // Equity – Share Classes & Shareholders (abbreviated for length;
    // follow the same pattern as above for the remaining methods)
    // -------------------------------------------------------------------------
    async fn create_share_class(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateShareClass,
    ) -> Result<ShareClass, AppError> {
        let row = sqlx::query_as::<_, ShareClass>(
            r#"
            INSERT INTO capital.share_classes (
                company_id, class_code, name, share_type,
                authorized_shares, issued_shares, outstanding_shares,
                par_value, currency_id, voting_rights, votes_per_share,
                dividend_rights, dividend_preference, liquidation_preference,
                is_callable, is_convertible, conversion_ratio
            )
            VALUES (
                $1,$2,$3,$4,
                $5,0,0,
                $6,$7,COALESCE($8,true),COALESCE($9,1),
                COALESCE($10,true),$11,$12,
                COALESCE($13,false),COALESCE($14,false),$15
            )
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(&payload.class_code)
        .bind(&payload.name)
        .bind(&payload.share_type)
        .bind(payload.authorized_shares)
        .bind(payload.par_value)
        .bind(payload.currency_id)
        .bind(payload.voting_rights)
        .bind(payload.votes_per_share)
        .bind(payload.dividend_rights)
        .bind(&payload.dividend_preference)
        .bind(payload.liquidation_preference)
        .bind(payload.is_callable)
        .bind(payload.is_convertible)
        .bind(payload.conversion_ratio)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn get_share_class(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<ShareClass, AppError> {
        sqlx::query_as::<_, ShareClass>(
            r#"
            SELECT * FROM capital.share_classes
            WHERE id = $1 AND company_id = $2
            "#,
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Share class not found".into()))
    }

    async fn list_share_classes(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<ShareClass>, AppError> {
        let rows = sqlx::query_as::<_, ShareClass>(
            r#"
            SELECT * FROM capital.share_classes
            WHERE company_id = $1
            ORDER BY class_code
            "#,
        )
        .bind(company_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn create_shareholder(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        company_id: Uuid,
        payload: &CreateShareholder,
    ) -> Result<Shareholder, AppError> {
        let row = sqlx::query_as::<_, Shareholder>(
            r#"
            INSERT INTO capital.shareholders (
                company_id, party_id, name, shareholder_type,
                tax_id, residency_country, is_related_party
            )
            VALUES ($1,$2,$3,$4,$5,$6,COALESCE($7,false))
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(payload.party_id)
        .bind(&payload.name)
        .bind(&payload.shareholder_type)
        .bind(&payload.tax_id)
        .bind(&payload.residency_country)
        .bind(payload.is_related_party)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row)
    }

    async fn list_shareholders(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<Shareholder>, AppError> {
        let rows = sqlx::query_as::<_, Shareholder>(
            r#"
            SELECT * FROM capital.shareholders
            WHERE company_id = $1
            ORDER BY name
            "#,
        )
        .bind(company_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn get_shareholding(
        &self,
        pool: &PgPool,
        share_class_id: Uuid,
        shareholder_id: Uuid,
    ) -> Result<Option<Shareholding>, AppError> {
        let row = sqlx::query_as::<_, Shareholding>(
            r#"
            SELECT * FROM capital.shareholdings
            WHERE share_class_id = $1 AND shareholder_id = $2
            "#,
        )
        .bind(share_class_id)
        .bind(shareholder_id)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    async fn create_share_transaction(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _user_id: Uuid,
        _payload: &CreateShareTransaction,
    ) -> Result<ShareTransaction, AppError> {
        let row = sqlx::query_as::<_, ShareTransaction>(
            r#"
            INSERT INTO capital.share_transactions
            WHERE id = $1
            transaction_type = $2, transaction_date = $3, from_shareholder_id = $4,
            to_shareholder_id = $5, shares = $6, price_per_share = $7,
            total_consideration = $8, currency_id = $9, premium = $10,
            journal_entry_id = $11, reference = $12, notes = $13,
            AND company_id = $14
            "#,
        )
        .bind(company_id)
        .bind(&_payload.share_class_id)
        .bind(_payload.transaction_type)
        .bind(_payload.transaction_date)
        .bind(_payload.from_shareholder_id)
        .bind(_payload.to_shareholder_id)
        .bind(_payload.shares)
        .bind(_payload.price_per_share)
        .bind(_payload.total_consideration)
        .bind(_payload.currency_id)
        .bind(_payload.premium)
        .bind(_payload.journal_entry_id)
        .bind(_payload.reference)
        .bind(_payload.notes)
        .fetch_one(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital instrument not found".into()));

        Ok(row)
    }

    async fn list_share_transactions(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _share_class_id: Option<Uuid>,
    ) -> Result<Vec<ShareTransaction>, AppError> {
         let rows = sqlx::query_as::<_, ShareTransaction>(
            r#"
            SELECT * FROM capital.share_transactions
            WHERE company_id = $1
            ORDER BY name
            "#,
        )
        .bind(company_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn create_dividend(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateDividend,
    ) -> Result<Dividend, AppError> {
         let row = sqlx::query_as::<_, Dividend>(
            r#"
            INSERT INTO capital.dividends
            WHERE id = $1
            dividend_type = $2,
            declaration_date = $3,
            record_date = $4,
            ex_dividend_date = $5,
            payment_date = $6,
            dividend_per_share = $7,
            total_declared = $8,
            currency_id = $9,
            status = $10,
            journal_entry_id = $11,
            created_at = $12,
            AND company_id = $1
            "#,
        )
        .bind(company_id)
        .bind(&_payload.share_class_id)
        .bind(_payload.dividend_type)
        .bind(_payload.declaration_date)
        .bind(_payload.record_date)
        .bind(_payload.ex_dividend_date)
        .bind(_payload.payment_date)
        .bind(_payload.dividend_per_share)
        .bind(_payload.total_declared)
        .bind(_payload.currency_id)
        .bind(_payload.status)
        .bind(_payload.journal_entry_id)
        .bind(_payload.created_at)
        .fetch_one(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("Capital instrument not found".into()));

        Ok(row)
    }

    async fn list_dividends(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _share_class_id: Option<Uuid>,
    ) -> Result<Vec<Dividend>, AppError> {
        todo!("Implement list_dividends")
    }

    async fn create_equity_account(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateEquityAccount,
    ) -> Result<EquityAccount, AppError> {
        todo!("Implement create_equity_account")
    }

    async fn list_equity_accounts(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
    ) -> Result<Vec<EquityAccount>, AppError> {
        todo!("Implement list_equity_accounts")
    }

    async fn create_equity_movement(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateEquityMovement,
    ) -> Result<EquityMovement, AppError> {
        todo!("Implement create_equity_movement")
    }

    async fn list_equity_movements(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _equity_account_id: Option<Uuid>,
        _from: Option<NaiveDate>,
        _to: Option<NaiveDate>,
    ) -> Result<Vec<EquityMovement>, AppError> {
        todo!("Implement list_equity_movements")
    }

    async fn create_structure_snapshot(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateCapitalStructureSnapshot,
    ) -> Result<CapitalStructureSnapshot, AppError> {
        todo!("Implement create_structure_snapshot")
    }

    async fn get_latest_structure(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
    ) -> Result<Option<CapitalStructureSnapshot>, AppError> {
        todo!("Implement get_latest_structure")
    }

    async fn create_allocation(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateCapitalAllocation,
    ) -> Result<CapitalAllocation, AppError> {
        todo!("Implement create_allocation")
    }

    async fn list_allocations(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
    ) -> Result<Vec<CapitalAllocation>, AppError> {
        todo!("Implement list_allocations")
    }

    async fn create_project(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateCapitalProject,
    ) -> Result<CapitalProject, AppError> {
        todo!("Implement create_project")
    }

    async fn get_project(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _id: Uuid,
    ) -> Result<CapitalProject, AppError> {
        todo!("Implement get_project")
    }

    async fn list_projects(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _status: Option<&str>,
    ) -> Result<Vec<CapitalProject>, AppError> {
        todo!("Implement list_projects")
    }

    async fn update_project(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _id: Uuid,
        _payload: &UpdateCapitalProject,
    ) -> Result<CapitalProject, AppError> {
        todo!("Implement update_project")
    }

    async fn add_project_funding(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _payload: &CreateProjectFunding,
    ) -> Result<ProjectFunding, AppError> {
        todo!("Implement add_project_funding")
    }

    async fn create_cash_forecast(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _user_id: Uuid,
        _payload: &CreateCashForecast,
    ) -> Result<CashForecast, AppError> {
        todo!("Implement create_cash_forecast")
    }

    async fn add_forecast_line(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _forecast_id: Uuid,
        _payload: &CreateCashForecastLine,
    ) -> Result<CashForecastLine, AppError> {
        todo!("Implement add_forecast_line")
    }

    async fn get_cash_forecast(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _id: Uuid,
    ) -> Result<CashForecast, AppError> {
        todo!("Implement get_cash_forecast")
    }

    async fn list_forecast_lines(
        &self,
        _pool: &PgPool,
        _forecast_id: Uuid,
    ) -> Result<Vec<CashForecastLine>, AppError> {
        todo!("Implement list_forecast_lines")
    }

    async fn record_metric(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateCapitalMetric,
    ) -> Result<CapitalMetric, AppError> {
        todo!("Implement record_metric")
    }

    async fn list_metrics(
        &self,
        _pool: &PgPool,
        _company_id: Uuid,
        _metric_code: Option<&str>,
        _from: Option<NaiveDate>,
        _to: Option<NaiveDate>,
    ) -> Result<Vec<CapitalMetric>, AppError> {
        todo!("Implement list_metrics")
    }

    async fn upsert_wacc(
        &self,
        _tx: &mut Transaction<'_, Postgres>,
        _company_id: Uuid,
        _payload: &CreateWaccComponent,
    ) -> Result<WaccComponent, AppError> {
        todo!("Implement upsert_wacc")
    }
}
