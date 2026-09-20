use crate::{
    features::investment::{
        metrics::{calc_discounted_payback, calc_irr, calc_mirr, calc_npv, calc_payback, calc_pi},
        repository::{ComputedMetrics, InvestmentRepository},
    },
    interface::api::errors::AppError,
    models::investment::*,
};
use bigdecimal::{BigDecimal, Zero};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub struct InvestmentService<R: InvestmentRepository> {
    repo: R,
}

impl<R: InvestmentRepository> InvestmentService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_program(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateInvestmentProgram,
    ) -> Result<InvestmentProgram, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;
        self.repo.create_program(pool, company_id, payload).await
    }

    pub async fn list_programs(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<InvestmentProgram>, AppError> {
        self.repo.list_programs(pool, company_id).await
    }

    pub async fn create_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
        payload: &CreateInvestmentCase,
    ) -> Result<InvestmentCase, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;
        self.repo
            .create_case(pool, company_id, user_id, payload)
            .await
    }

    pub async fn get_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<InvestmentCase, AppError> {
        self.repo.get_case(pool, company_id, id).await
    }

    pub async fn list_cases(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        stage: Option<&str>,
    ) -> Result<Vec<InvestmentCase>, AppError> {
        self.repo.list_cases(pool, company_id, stage).await
    }

    pub async fn update_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateInvestmentCase,
    ) -> Result<InvestmentCase, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;
        self.repo.update_case(pool, company_id, id, payload).await
    }

    pub async fn advance_stage(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        case_id: Uuid,
        user_id: Uuid,
        payload: &AdvanceStageRequest,
    ) -> Result<InvestmentCase, AppError> {
        let _ = self.repo.get_case(pool, company_id, case_id).await?;
        self.repo
            .advance_case(
                pool,
                case_id,
                &payload.to_stage,
                user_id,
                payload.comment.as_deref(),
            )
            .await
    }

    pub async fn create_scenario(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        case_id: Uuid,
        payload: &CreateInvestmentScenario,
    ) -> Result<InvestmentScenario, AppError> {
        let _ = self.repo.get_case(pool, company_id, case_id).await?;
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;
        self.repo.create_scenario(pool, case_id, payload).await
    }

    pub async fn list_scenarios(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentScenario>, AppError> {
        self.repo.list_scenarios(pool, case_id).await
    }

    pub async fn add_cashflow(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
        payload: &CreateCashflowLine,
    ) -> Result<InvestmentCashflowLine, AppError> {
        payload
            .validate()
            .map_err(|e| AppError::Unprocessable(e.to_string()))?;
        self.repo
            .add_cashflow_line(pool, scenario_id, payload)
            .await
    }

    pub async fn list_cashflows(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
    ) -> Result<Vec<InvestmentCashflowLine>, AppError> {
        self.repo.list_cashflow_lines(pool, scenario_id).await
    }

    /// Core appraisal: load period CFs → compute metrics → persist
    pub async fn recalculate_metrics(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        case_id: Uuid,
        scenario_id: Uuid,
        user_id: Uuid,
        req: &RecalculateMetricsRequest,
    ) -> Result<InvestmentMetrics, AppError> {
        let case = self.repo.get_case(pool, company_id, case_id).await?;
        let rate = req
            .discount_rate
            .clone()
            .or(case.hurdle_rate.clone())
            .ok_or_else(|| {
                AppError::BadRequest("discount_rate or case.hurdle_rate required".into())
            })?;

        let periods = self.repo.period_net_cashflows(pool, scenario_id).await?;
        if periods.is_empty() {
            return Err(AppError::BadRequest(
                "No cash flows for scenario — add lines first".into(),
            ));
        }

        // Dense array 0..max_period
        let max_p = periods.iter().map(|p| p.period_no).max().unwrap_or(0) as usize;
        let mut cfs = vec![BigDecimal::zero(); max_p + 1];
        for p in &periods {
            let idx = p.period_no as usize;
            cfs[idx] = p.net_cash.clone();
        }

        let reinvest = req.reinvest_rate.clone().unwrap_or_else(|| rate.clone());
        let lines = self.repo.list_cashflow_lines(pool, scenario_id).await?;
        let total_capex: BigDecimal = lines
            .iter()
            .filter(|l| l.line_type == "CAPEX")
            .map(|l| l.amount.abs())
            .sum();

        let computed = ComputedMetrics {
            npv: Some(calc_npv(&rate, &cfs)),
            irr: calc_irr(&cfs),
            mirr: calc_mirr(&cfs, &rate, &reinvest),
            payback_years: calc_payback(&cfs),
            discounted_payback: calc_discounted_payback(&rate, &cfs),
            profitability_index: calc_pi(&rate, &cfs),
            total_capex: Some(total_capex),
        };

        let metrics = self
            .repo
            .insert_metrics(pool, scenario_id, &rate, &computed, user_id)
            .await?;

        // Cache on case when scenario is primary
        let scenarios = self.repo.list_scenarios(pool, case_id).await?;
        if scenarios
            .iter()
            .any(|s| s.id == scenario_id && s.is_primary)
        {
            self.repo
                .update_case_metrics_cache(pool, case_id, &computed)
                .await?;
        }

        Ok(metrics)
    }

    pub async fn create_risk(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        case_id: Uuid,
        payload: &CreateInvestmentRisk,
    ) -> Result<InvestmentRisk, AppError> {
        let _ = self.repo.get_case(pool, company_id, case_id).await?;
        self.repo.create_risk(pool, case_id, payload).await
    }

    pub async fn list_risks(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentRisk>, AppError> {
        self.repo.list_risks(pool, case_id).await
    }

    pub async fn post_audit(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        case_id: Uuid,
        user_id: Uuid,
        payload: &CreatePostAudit,
    ) -> Result<InvestmentPostAudit, AppError> {
        let case = self.repo.get_case(pool, company_id, case_id).await?;
        self.repo
            .create_post_audit(
                pool,
                case_id,
                user_id,
                case.npv.clone(),
                case.irr.clone(),
                payload,
            )
            .await
    }

    pub async fn portfolio_board(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<PortfolioBoardRow>, AppError> {
        self.repo.portfolio_board(pool, company_id).await
    }
}
