// src/features/capital/services.rs
use crate::{
    features::capital::repository::{CapitalRepository, gl_line, post_gl},
    interface::api::errors::AppError,
    models::capital::*,
};

use bigdecimal::{BigDecimal, Zero};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub struct CapitalService<R: CapitalRepository> {
    repo: R,
}

impl<R: CapitalRepository> CapitalService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // =========================================================================
    // 1. Parties
    // =========================================================================

    pub async fn create_party(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateParty,
    ) -> Result<Party, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let party = self
            .repo
            .create_party(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(party)
    }

    pub async fn get_party(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<Party, AppError> {
        self.repo.get_party(pool, company_id, id).await
    }

    pub async fn list_parties(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        uuid: Uuid,
    ) -> Result<Vec<Party>, AppError> {
        self.repo.list_parties(pool, company_id, uuid).await
    }

    pub async fn update_party(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &CreateParty,
    ) -> Result<Party, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let party = self
            .repo
            .update_party(&mut tx, company_id, id, payload)
            .await?;
        tx.commit().await?;
        Ok(party)
    }

    // =========================================================================
    // 1. Capital Instruments
    // =========================================================================

    pub async fn create_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let instrument = self
            .repo
            .create_instrument(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(instrument)
    }

    pub async fn get_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalInstrument, AppError> {
        self.repo.get_instrument(pool, company_id, id).await
    }

    pub async fn list_instruments(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        family: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<CapitalInstrument>, AppError> {
        self.repo
            .list_instruments(pool, company_id, family, status)
            .await
    }

    pub async fn update_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalInstrument,
    ) -> Result<CapitalInstrument, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let instrument = self
            .repo
            .update_instrument(&mut tx, company_id, id, payload)
            .await?;
        tx.commit().await?;
        Ok(instrument)
    }

    // =========================================================================
    // 2. Capital Events
    // =========================================================================

    pub async fn create_event(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalEvent,
    ) -> Result<CapitalEvent, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let mut event = self
            .repo
            .create_event(&mut tx, company_id, user_id, payload)
            .await?;

        // Post to gl if capital contribution
        if payload.event_type == "EQUITY_CONTRIBUTION" {
            let journal_id = if let (Some(capital), Some(cash), Some(amount), txn_date) = (
                &payload.capital_contrib_acc,
                &payload.cash_account,
                payload.amount.clone(),
                &payload.event_date,
            ) {
                if amount > BigDecimal::zero() {
                    let lines = vec![
                        gl_line(
                            *cash,
                            amount.clone(),
                            BigDecimal::zero(),
                            "Cash or bank increament",
                        ),
                        gl_line(
                            *capital,
                            BigDecimal::zero(),
                            amount.clone(),
                            "Capital contribution",
                        ),
                    ];
                    Some(
                        post_gl(
                            &mut tx,
                            &format!("CAP-CONTRIB-{}", &event.serial_id),
                            "Capital contribution",
                            user_id,
                            *txn_date,
                            lines,
                        )
                        .await?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

            event = self
                .repo
                .update_event_journal_id(&mut tx, event.id, journal_id)
                .await?;
        }

        tx.commit().await?;
        Ok(event)
    }

    pub async fn list_events_for_instrument(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        instrument_id: Uuid,
    ) -> Result<Vec<CapitalEvent>, AppError> {
        self.repo
            .list_events_for_instrument(pool, company_id, instrument_id)
            .await
    }

    // =========================================================================
    // 3. Debt Facilities
    // =========================================================================

    pub async fn create_facility(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCapitalFacility,
    ) -> Result<CapitalFacility, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.committed_amount <= BigDecimal::zero() {
            return Err(AppError::Unprocessable(
                "committed_amount must be greater than zero".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let facility = self
            .repo
            .create_facility(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(facility)
    }

    pub async fn get_facility(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError> {
        self.repo.get_facility(pool, company_id, id).await
    }

    pub async fn list_facilities(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<CapitalFacility>, AppError> {
        self.repo.list_facilities(pool, company_id, status).await
    }

    pub async fn update_facility(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalFacility,
    ) -> Result<CapitalFacility, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let facility = self
            .repo
            .update_facility(&mut tx, company_id, id, payload)
            .await?;
        tx.commit().await?;
        Ok(facility)
    }

    pub async fn add_facility_lender(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
        lender_id: Uuid,
        commitment_amount: BigDecimal,
        participation_pct: Option<BigDecimal>,
        is_agent: bool,
    ) -> Result<FacilityLender, AppError> {
        if commitment_amount <= BigDecimal::zero() {
            return Err(AppError::Unprocessable(
                "commitment_amount must be greater than zero".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let lender = self
            .repo
            .add_facility_lender(
                &mut tx,
                facility_id,
                lender_id,
                commitment_amount,
                participation_pct,
                is_agent,
            )
            .await?;
        tx.commit().await?;
        Ok(lender)
    }

    pub async fn list_facility_lenders(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<FacilityLender>, AppError> {
        self.repo.list_facility_lenders(pool, facility_id).await
    }

    // =========================================================================
    // 4. Drawdowns
    // =========================================================================

    pub async fn create_drawdown(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateDrawdown,
    ) -> Result<DebtDrawdown, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.amount <= BigDecimal::zero() {
            return Err(AppError::Unprocessable(
                "drawdown amount must be greater than zero".into(),
            ));
        }

        // Business rule: ensure facility has enough available headroom
        let facility = self
            .repo
            .get_facility(pool, company_id, payload.facility_id)
            .await?;

        if payload.amount > facility.available_amount {
            return Err(AppError::Unprocessable(format!(
                "Insufficient available amount. Requested {}, available {}",
                payload.amount, facility.available_amount
            )));
        }

        let mut tx = pool.begin().await?;
        let drawdown = self
            .repo
            .create_drawdown(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(drawdown)
    }

    pub async fn list_drawdowns(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError> {
        self.repo.list_drawdowns(pool, facility_id).await
    }

    // =========================================================================
    // 5. Repayment Schedules & Repayments
    // =========================================================================

    pub async fn create_repayment_schedule(
        &self,
        pool: &PgPool,
        payload: &CreateRepaymentSchedule,
    ) -> Result<DebtRepaymentSchedule, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let schedule = self
            .repo
            .create_repayment_schedule(&mut tx, payload)
            .await?;
        tx.commit().await?;
        Ok(schedule)
    }

    pub async fn list_repayment_schedules(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepaymentSchedule>, AppError> {
        self.repo.list_repayment_schedules(pool, facility_id).await
    }

    pub async fn create_repayment(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateRepayment,
    ) -> Result<DebtRepayment, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.principal_amount < BigDecimal::zero()
            || payload.interest_amount < BigDecimal::zero()
        {
            return Err(AppError::Unprocessable(
                "principal and interest amounts cannot be negative".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let repayment = self
            .repo
            .create_repayment(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(repayment)
    }

    pub async fn list_repayments(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError> {
        self.repo.list_repayments(pool, facility_id).await
    }

    // =========================================================================
    // 6. Interest Accruals & Fees
    // =========================================================================

    pub async fn create_interest_accrual(
        &self,
        pool: &PgPool,
        payload: &CreateInterestAccrual,
        user_id: Uuid,
    ) -> Result<DebtInterestAccrual, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.period_end < payload.period_start {
            return Err(AppError::Unprocessable(
                "period_end must be on or after period_start".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let accrual = self
            .repo
            .create_interest_accrual(&mut tx, payload, user_id)
            .await?;
        tx.commit().await?;
        Ok(accrual)
    }

    pub async fn list_interest_accruals(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtInterestAccrual>, AppError> {
        self.repo.list_interest_accruals(pool, facility_id).await
    }

    pub async fn create_debt_fee(
        &self,
        pool: &PgPool,
        payload: &CreateDebtFee,
    ) -> Result<DebtFee, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let fee = self.repo.create_debt_fee(&mut tx, payload).await?;
        tx.commit().await?;
        Ok(fee)
    }

    // =========================================================================
    // 7. Covenants
    // =========================================================================

    pub async fn create_covenant(
        &self,
        pool: &PgPool,
        payload: &CreateDebtCovenant,
    ) -> Result<DebtCovenant, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let covenant = self.repo.create_covenant(&mut tx, payload).await?;
        tx.commit().await?;
        Ok(covenant)
    }

    pub async fn list_covenants(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtCovenant>, AppError> {
        self.repo.list_covenants(pool, facility_id).await
    }

    pub async fn record_covenant_test(
        &self,
        pool: &PgPool,
        covenant_id: Uuid,
        test_date: NaiveDate,
        actual_value: Option<BigDecimal>,
        is_compliant: Option<bool>,
        headroom: Option<BigDecimal>,
        notes: String,
        tested_by: Uuid,
    ) -> Result<DebtCovenantTest, AppError> {
        let mut tx = pool.begin().await?;
        let test = self
            .repo
            .record_covenant_test(
                &mut tx,
                covenant_id,
                test_date,
                actual_value,
                is_compliant,
                headroom,
                notes,
                tested_by,
            )
            .await?;
        tx.commit().await?;
        Ok(test)
    }

    // =========================================================================
    // 8. Collateral & Refinancing
    // =========================================================================

    pub async fn create_collateral(
        &self,
        pool: &PgPool,
        payload: &CreateDebtCollateral,
    ) -> Result<DebtCollateral, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let collateral = self.repo.create_collateral(&mut tx, payload).await?;
        tx.commit().await?;
        Ok(collateral)
    }

    pub async fn create_refinancing(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateDebtRefinancing,
    ) -> Result<DebtRefinancing, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let refinancing = self
            .repo
            .create_refinancing(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(refinancing)
    }

    // =========================================================================
    // 9. Equity – Share Classes & Shareholders
    // =========================================================================

    pub async fn create_share_class(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateShareClass,
    ) -> Result<ShareClass, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let share_class = self
            .repo
            .create_share_class(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(share_class)
    }

    pub async fn get_share_class(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<ShareClass, AppError> {
        self.repo.get_share_class(pool, company_id, id).await
    }

    pub async fn list_share_classes(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<ShareClass>, AppError> {
        self.repo.list_share_classes(pool, company_id).await
    }

    pub async fn create_shareholder(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateShareholder,
    ) -> Result<Shareholder, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let shareholder = self
            .repo
            .create_shareholder(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(shareholder)
    }

    pub async fn list_shareholders(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<Shareholder>, AppError> {
        self.repo.list_shareholders(pool, company_id).await
    }

    pub async fn get_shareholding(
        &self,
        pool: &PgPool,
        share_class_id: Uuid,
        shareholder_id: Uuid,
    ) -> Result<Option<Shareholding>, AppError> {
        self.repo
            .get_shareholding(pool, share_class_id, shareholder_id)
            .await
    }

    // =========================================================================
    // 10. Share Transactions & Dividends
    // =========================================================================

    pub async fn create_share_transaction(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateShareTransaction,
    ) -> Result<ShareTransaction, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.shares <= 0 {
            return Err(AppError::Unprocessable(
                "shares must be greater than zero".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let tx_row = self
            .repo
            .create_share_transaction(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(tx_row)
    }

    pub async fn list_share_transactions(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        share_class_id: Option<Uuid>,
    ) -> Result<Vec<ShareTransaction>, AppError> {
        self.repo
            .list_share_transactions(pool, company_id, share_class_id)
            .await
    }

    pub async fn create_dividend(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateDividend,
        user_id: Uuid,
    ) -> Result<Dividend, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.dividend_per_share < BigDecimal::zero() {
            return Err(AppError::Unprocessable(
                "dividend_per_share cannot be negative".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let dividend = self
            .repo
            .create_dividend(&mut tx, company_id, payload, user_id)
            .await?;
        tx.commit().await?;
        Ok(dividend)
    }

    pub async fn list_dividends(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        share_class_id: Option<Uuid>,
    ) -> Result<Vec<Dividend>, AppError> {
        self.repo
            .list_dividends(pool, company_id, share_class_id)
            .await
    }

    /// Generate payments for all current holders and mark dividend as paid.
    pub async fn pay_dividend(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        req: &PayDividendRequest,
        dividend_payable_account: Option<Uuid>,
        cash_account: Option<Uuid>,
    ) -> Result<(Dividend, Vec<DividendPayment>), AppError> {
        req.validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        // 1. Load dividend
        let dividend = self
            .repo
            .list_dividends(pool, company_id, None)
            .await?
            .into_iter()
            .find(|d| d.id == req.dividend_id)
            .ok_or_else(|| AppError::NotFound("Dividend not found".into()))?;

        if dividend.status == "paid" {
            return Err(AppError::Unprocessable("Dividend is already paid".into()));
        }

        // 2. Holdings for that share class
        let holdings = self
            .repo
            .list_shareholdings_for_dividend(pool, company_id, dividend.share_class_id)
            .await?;

        if holdings.is_empty() {
            return Err(AppError::Unprocessable(
                "No shareholdings found for this share class".into(),
            ));
        }

        let rate = req.withholding_rate.clone().unwrap_or(BigDecimal::zero());
        let mut tx = pool.begin().await?;
        let mut payments = Vec::with_capacity(holdings.len());
        let mut total_net = BigDecimal::zero();

        // 3. One payment row per holder
        for h in &holdings {
            let gross = dividend.dividend_per_share.clone() * BigDecimal::from(h.shares_held);
            let withholding = (gross.clone() * rate.clone()).round(2);
            let net = gross.clone() - withholding.clone();
            total_net += net;

            let reference = req
                .payment_reference_prefix
                .as_ref()
                .map(|p| format!("{}-{}", p, h.shareholder_id));

            let payload = CreateDividendPayment {
                dividend_id: dividend.id,
                shareholder_id: h.shareholder_id,
                shares_held: h.shares_held,
                gross_amount: gross.clone(),
                withholding_tax: Some(withholding.clone()),
                payment_date: Some(req.payment_date),
                payment_reference: reference,
                journal_entry_id: None,
            };

            let payment = self.repo.create_dividend_payment(&mut tx, &payload).await?;
            payments.push(payment);
        }

        // 4. Optional GL: Dr Dividend Payable, Cr Cash (net)
        //    (Withholding would credit a tax payable account if you track it)
        let journal_id =
            if let (Some(payable), Some(cash)) = (dividend_payable_account, cash_account) {
                if total_net > BigDecimal::zero() {
                    let lines = vec![
                        gl_line(
                            payable,
                            total_net.clone(),
                            BigDecimal::zero(),
                            "Dividend payable settlement",
                        ),
                        gl_line(
                            cash,
                            BigDecimal::zero(),
                            total_net.clone(),
                            &format!("Dividend payment – {}", dividend.id),
                        ),
                    ];
                    Some(
                        post_gl(
                            &mut tx,
                            &format!("CAP-DIV-PAY-{}", dividend.id),
                            &format!("Dividend payment"),
                            user_id,
                            req.payment_date,
                            lines,
                        )
                        .await?,
                    )
                } else {
                    None
                }
            } else {
                None
            };

        // 5. Mark dividend paid
        let paid = self
            .repo
            .mark_dividend_paid(
                &mut tx,
                company_id,
                dividend.id,
                req.payment_date,
                journal_id,
            )
            .await?;

        tx.commit().await?;
        Ok((paid, payments))
    }

    pub async fn list_dividend_payments(
        &self,
        pool: &PgPool,
        dividend_id: Uuid,
    ) -> Result<Vec<DividendPayment>, AppError> {
        self.repo.list_dividend_payments(pool, dividend_id).await
    }

    // =========================================================================
    // 11. Equity Accounts & Movements (Retained Earnings)
    // =========================================================================

    pub async fn create_equity_account(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateEquityAccount,
    ) -> Result<EquityAccount, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let account = self
            .repo
            .create_equity_account(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(account)
    }

    pub async fn list_equity_accounts(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<EquityAccount>, AppError> {
        self.repo.list_equity_accounts(pool, company_id).await
    }

    pub async fn create_equity_movement(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateEquityMovement,
    ) -> Result<EquityMovement, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let movement = self
            .repo
            .create_equity_movement(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(movement)
    }

    pub async fn list_equity_movements(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        equity_account_id: Option<Uuid>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<EquityMovement>, AppError> {
        self.repo
            .list_equity_movements(pool, company_id, equity_account_id, from, to)
            .await
    }

    // =========================================================================
    // 12. Capital Structure & Allocations
    // =========================================================================

    pub async fn create_structure_snapshot(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateCapitalStructureSnapshot,
    ) -> Result<CapitalStructureSnapshot, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let snapshot = self
            .repo
            .create_structure_snapshot(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(snapshot)
    }

    pub async fn get_latest_structure(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Option<CapitalStructureSnapshot>, AppError> {
        self.repo.get_latest_structure(pool, company_id).await
    }

    pub async fn create_allocation(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateCapitalAllocation,
    ) -> Result<CapitalAllocation, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        if payload.amount <= BigDecimal::zero() {
            return Err(AppError::Unprocessable(
                "allocation amount must be greater than zero".into(),
            ));
        }

        let mut tx = pool.begin().await?;
        let allocation = self
            .repo
            .create_allocation(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(allocation)
    }

    pub async fn list_allocations(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<CapitalAllocation>, AppError> {
        self.repo.list_allocations(pool, company_id).await
    }

    // =========================================================================
    // 13. Projects
    // =========================================================================

    pub async fn create_project(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateCapitalProject,
    ) -> Result<CapitalProject, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let project = self
            .repo
            .create_project(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(project)
    }

    pub async fn get_project(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CapitalProject, AppError> {
        self.repo.get_project(pool, company_id, id).await
    }

    pub async fn list_projects(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<CapitalProject>, AppError> {
        self.repo.list_projects(pool, company_id, status).await
    }

    pub async fn update_project(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateCapitalProject,
    ) -> Result<CapitalProject, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let project = self
            .repo
            .update_project(&mut tx, company_id, id, payload)
            .await?;
        tx.commit().await?;
        Ok(project)
    }

    pub async fn add_project_funding(
        &self,
        pool: &PgPool,
        payload: &CreateProjectFunding,
    ) -> Result<ProjectFunding, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let funding = self.repo.add_project_funding(&mut tx, payload).await?;
        tx.commit().await?;
        Ok(funding)
    }

    // =========================================================================
    // 14. Liquidity / Cash Forecasts
    // =========================================================================

    pub async fn create_cash_forecast(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateCashForecast,
    ) -> Result<CashForecast, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let forecast = self
            .repo
            .create_cash_forecast(&mut tx, company_id, user_id, payload)
            .await?;
        tx.commit().await?;
        Ok(forecast)
    }

    pub async fn add_forecast_line(
        &self,
        pool: &PgPool,
        forecast_id: Uuid,
        payload: &CreateCashForecastLine,
    ) -> Result<CashForecastLine, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let line = self
            .repo
            .add_forecast_line(&mut tx, forecast_id, payload)
            .await?;
        tx.commit().await?;
        Ok(line)
    }

    pub async fn get_cash_forecast(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<CashForecast, AppError> {
        self.repo.get_cash_forecast(pool, company_id, id).await
    }

    pub async fn list_forecast_lines(
        &self,
        pool: &PgPool,
        forecast_id: Uuid,
    ) -> Result<Vec<CashForecastLine>, AppError> {
        self.repo.list_forecast_lines(pool, forecast_id).await
    }

    // =========================================================================
    // 15. Analytics
    // =========================================================================

    pub async fn record_metric(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateCapitalMetric,
    ) -> Result<CapitalMetric, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let metric = self
            .repo
            .record_metric(&mut tx, company_id, payload)
            .await?;
        tx.commit().await?;
        Ok(metric)
    }

    pub async fn list_metrics(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        metric_code: Option<&str>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CapitalMetric>, AppError> {
        self.repo
            .list_metrics(pool, company_id, metric_code, from, to)
            .await
    }

    pub async fn upsert_wacc(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateWaccComponent,
    ) -> Result<WaccComponent, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;

        let mut tx = pool.begin().await?;
        let wacc = self.repo.upsert_wacc(&mut tx, company_id, payload).await?;
        tx.commit().await?;
        Ok(wacc)
    }
}
