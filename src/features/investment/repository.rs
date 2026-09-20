use crate::{
    interface::api::errors::AppError,
    models::investment::*,
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait InvestmentRepository: Send + Sync {
    // Programs
    async fn create_program(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateInvestmentProgram,
    ) -> Result<InvestmentProgram, AppError>;
    async fn list_programs(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<InvestmentProgram>, AppError>;

    // Cases
    async fn create_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        initiator_id: Uuid,
        payload: &CreateInvestmentCase,
    ) -> Result<InvestmentCase, AppError>;
    async fn get_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<InvestmentCase, AppError>;
    async fn list_cases(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        stage: Option<&str>,
    ) -> Result<Vec<InvestmentCase>, AppError>;
    async fn update_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateInvestmentCase,
    ) -> Result<InvestmentCase, AppError>;
    async fn advance_case(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        to_stage: &str,
        actor_id: Uuid,
        comment: Option<&str>,
    ) -> Result<InvestmentCase, AppError>;

    // Scenarios & CF
    async fn create_scenario(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        payload: &CreateInvestmentScenario,
    ) -> Result<InvestmentScenario, AppError>;
    async fn list_scenarios(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentScenario>, AppError>;
    async fn add_cashflow_line(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
        payload: &CreateCashflowLine,
    ) -> Result<InvestmentCashflowLine, AppError>;
    async fn list_cashflow_lines(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
    ) -> Result<Vec<InvestmentCashflowLine>, AppError>;
    async fn period_net_cashflows(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
    ) -> Result<Vec<PeriodCashflow>, AppError>;

    async fn insert_metrics(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
        discount_rate: &BigDecimal,
        metrics: &ComputedMetrics,
        user_id: Uuid,
    ) -> Result<InvestmentMetrics, AppError>;

    async fn update_case_metrics_cache(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        metrics: &ComputedMetrics,
    ) -> Result<(), AppError>;

    // Risks / post-audit / board
    async fn create_risk(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        payload: &CreateInvestmentRisk,
    ) -> Result<InvestmentRisk, AppError>;
    async fn list_risks(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentRisk>, AppError>;
    async fn create_post_audit(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        user_id: Uuid,
        promised_npv: Option<BigDecimal>,
        promised_irr: Option<BigDecimal>,
        payload: &CreatePostAudit,
    ) -> Result<InvestmentPostAudit, AppError>;
    async fn portfolio_board(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<PortfolioBoardRow>, AppError>;
}

/// Result of Rust-side metric engine
#[derive(Debug, Clone, Default)]
pub struct ComputedMetrics {
    pub npv: Option<BigDecimal>,
    pub irr: Option<BigDecimal>,
    pub mirr: Option<BigDecimal>,
    pub payback_years: Option<BigDecimal>,
    pub discounted_payback: Option<BigDecimal>,
    pub profitability_index: Option<BigDecimal>,
    pub total_capex: Option<BigDecimal>,
}

pub struct PostgresInvestmentRepo;

impl PostgresInvestmentRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InvestmentRepository for PostgresInvestmentRepo {
    async fn create_program(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        payload: &CreateInvestmentProgram,
    ) -> Result<InvestmentProgram, AppError> {
        sqlx::query_as::<_, InvestmentProgram>(
            r#"
            INSERT INTO capital.investment_programs (
                company_id, program_code, name, description,
                horizon_start, horizon_end, owner_id
            ) VALUES ($1,$2,$3,$4,$5,$6,$7)
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(&payload.program_code)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(payload.horizon_start)
        .bind(payload.horizon_end)
        .bind(payload.owner_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn list_programs(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<InvestmentProgram>, AppError> {
        sqlx::query_as::<_, InvestmentProgram>(
            r#"
            SELECT * FROM capital.investment_programs
            WHERE company_id = $1 ORDER BY program_code
            "#,
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn create_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        initiator_id: Uuid,
        payload: &CreateInvestmentCase,
    ) -> Result<InvestmentCase, AppError> {
        sqlx::query_as::<_, InvestmentCase>(
            r#"
            INSERT INTO capital.investment_cases (
                company_id, case_code, title, description, investment_type,
                program_id, strategic_score, strategic_notes, risk_rating,
                currency_id, hurdle_rate, initiator_id, stage
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,'IDEA')
            RETURNING *
            "#,
        )
        .bind(company_id)
        .bind(&payload.case_code)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.investment_type)
        .bind(payload.program_id)
        .bind(&payload.strategic_score)
        .bind(&payload.strategic_notes)
        .bind(&payload.risk_rating)
        .bind(payload.currency_id)
        .bind(&payload.hurdle_rate)
        .bind(initiator_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn get_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
    ) -> Result<InvestmentCase, AppError> {
        sqlx::query_as::<_, InvestmentCase>(
            r#"SELECT * FROM capital.investment_cases WHERE id = $1 AND company_id = $2"#,
        )
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Investment case not found".into()))
    }

    async fn list_cases(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        stage: Option<&str>,
    ) -> Result<Vec<InvestmentCase>, AppError> {
        sqlx::query_as::<_, InvestmentCase>(
            r#"
            SELECT * FROM capital.investment_cases
            WHERE company_id = $1
              AND ($2::text IS NULL OR stage = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(company_id)
        .bind(stage)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn update_case(
        &self,
        pool: &PgPool,
        company_id: Uuid,
        id: Uuid,
        payload: &UpdateInvestmentCase,
    ) -> Result<InvestmentCase, AppError> {
        sqlx::query_as::<_, InvestmentCase>(
            r#"
            UPDATE capital.investment_cases SET
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                investment_type = COALESCE($5, investment_type),
                program_id = COALESCE($6, program_id),
                strategic_score = COALESCE($7, strategic_score),
                strategic_notes = COALESCE($8, strategic_notes),
                risk_rating = COALESCE($9, risk_rating),
                hurdle_rate = COALESCE($10, hurdle_rate),
                updated_at = now()
            WHERE id = $1 AND company_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.investment_type)
        .bind(payload.program_id)
        .bind(&payload.strategic_score)
        .bind(&payload.strategic_notes)
        .bind(&payload.risk_rating)
        .bind(&payload.hurdle_rate)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Investment case not found".into()))
    }

    async fn advance_case(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        to_stage: &str,
        actor_id: Uuid,
        comment: Option<&str>,
    ) -> Result<InvestmentCase, AppError> {
        // Prefer SQL function if present
        let row = sqlx::query_as::<_, InvestmentCase>(
            r#"SELECT * FROM capital.advance_investment_case($1, $2, $3, $4)"#,
        )
        .bind(case_id)
        .bind(to_stage)
        .bind(actor_id)
        .bind(comment)
        .fetch_one(pool)
        .await;

        match row {
            Ok(c) => Ok(c),
            Err(_) => {
                // Fallback without SQL function
                sqlx::query(
                    r#"
                    INSERT INTO capital.investment_approvals (case_id, stage, actor_id, action, comment)
                    VALUES ($1, $2, $3, 'APPROVE', $4)
                    "#,
                )
                .bind(case_id)
                .bind(to_stage)
                .bind(actor_id)
                .bind(comment)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;

                sqlx::query_as::<_, InvestmentCase>(
                    r#"
                    UPDATE capital.investment_cases
                    SET stage = $2, stage_changed_at = now(), updated_at = now()
                    WHERE id = $1
                    RETURNING *
                    "#,
                )
                .bind(case_id)
                .bind(to_stage)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)
            }
        }
    }

    async fn create_scenario(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        payload: &CreateInvestmentScenario,
    ) -> Result<InvestmentScenario, AppError> {
        sqlx::query_as::<_, InvestmentScenario>(
            r#"
            INSERT INTO capital.investment_scenarios (
                case_id, scenario_code, name, probability, inflation_rate, is_primary
            ) VALUES ($1,$2,$3,$4,$5,COALESCE($6,false))
            RETURNING *
            "#,
        )
        .bind(case_id)
        .bind(&payload.scenario_code)
        .bind(&payload.name)
        .bind(&payload.probability)
        .bind(&payload.inflation_rate)
        .bind(payload.is_primary)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn list_scenarios(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentScenario>, AppError> {
        sqlx::query_as::<_, InvestmentScenario>(
            r#"SELECT * FROM capital.investment_scenarios WHERE case_id = $1 ORDER BY scenario_code"#,
        )
        .bind(case_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn add_cashflow_line(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
        payload: &CreateCashflowLine,
    ) -> Result<InvestmentCashflowLine, AppError> {
        sqlx::query_as::<_, InvestmentCashflowLine>(
            r#"
            INSERT INTO capital.investment_cashflow_lines (
                scenario_id, period_no, period_date, line_type, description,
                amount, is_cash, tax_deductible
            ) VALUES ($1,$2,$3,$4,$5,$6,COALESCE($7,true),COALESCE($8,false))
            RETURNING *
            "#,
        )
        .bind(scenario_id)
        .bind(payload.period_no)
        .bind(payload.period_date)
        .bind(&payload.line_type)
        .bind(&payload.description)
        .bind(&payload.amount)
        .bind(payload.is_cash)
        .bind(payload.tax_deductible)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn list_cashflow_lines(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
    ) -> Result<Vec<InvestmentCashflowLine>, AppError> {
        sqlx::query_as::<_, InvestmentCashflowLine>(
            r#"
            SELECT * FROM capital.investment_cashflow_lines
            WHERE scenario_id = $1
            ORDER BY period_no, line_type
            "#,
        )
        .bind(scenario_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn period_net_cashflows(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
    ) -> Result<Vec<PeriodCashflow>, AppError> {
        sqlx::query_as::<_, PeriodCashflow>(
            r#"
            SELECT period_no, net_cash
            FROM capital.v_scenario_period_cashflows
            WHERE scenario_id = $1
            ORDER BY period_no
            "#,
        )
        .bind(scenario_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn insert_metrics(
        &self,
        pool: &PgPool,
        scenario_id: Uuid,
        discount_rate: &BigDecimal,
        metrics: &ComputedMetrics,
        user_id: Uuid,
    ) -> Result<InvestmentMetrics, AppError> {
        sqlx::query_as::<_, InvestmentMetrics>(
            r#"
            INSERT INTO capital.investment_metrics (
                scenario_id, discount_rate, npv, irr, mirr,
                payback_years, discounted_payback, profitability_index,
                total_capex, calculated_by
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *
            "#,
        )
        .bind(scenario_id)
        .bind(discount_rate)
        .bind(&metrics.npv)
        .bind(&metrics.irr)
        .bind(&metrics.mirr)
        .bind(&metrics.payback_years)
        .bind(&metrics.discounted_payback)
        .bind(&metrics.profitability_index)
        .bind(&metrics.total_capex)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn update_case_metrics_cache(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        metrics: &ComputedMetrics,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE capital.investment_cases SET
                npv = $2, irr = $3, mirr = $4,
                payback_years = $5, discounted_payback = $6,
                profitability_index = $7,
                metrics_calculated_at = now(), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(case_id)
        .bind(&metrics.npv)
        .bind(&metrics.irr)
        .bind(&metrics.mirr)
        .bind(&metrics.payback_years)
        .bind(&metrics.discounted_payback)
        .bind(&metrics.profitability_index)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;
        Ok(())
    }

    async fn create_risk(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        payload: &CreateInvestmentRisk,
    ) -> Result<InvestmentRisk, AppError> {
        sqlx::query_as::<_, InvestmentRisk>(
            r#"
            INSERT INTO capital.investment_risks (
                case_id, risk_code, title, category, likelihood, impact, mitigation, owner_id
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(case_id)
        .bind(&payload.risk_code)
        .bind(&payload.title)
        .bind(&payload.category)
        .bind(&payload.likelihood)
        .bind(&payload.impact)
        .bind(&payload.mitigation)
        .bind(payload.owner_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn list_risks(
        &self,
        pool: &PgPool,
        case_id: Uuid,
    ) -> Result<Vec<InvestmentRisk>, AppError> {
        sqlx::query_as::<_, InvestmentRisk>(
            r#"SELECT * FROM capital.investment_risks WHERE case_id = $1 ORDER BY created_at"#,
        )
        .bind(case_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    async fn create_post_audit(
        &self,
        pool: &PgPool,
        case_id: Uuid,
        user_id: Uuid,
        promised_npv: Option<BigDecimal>,
        promised_irr: Option<BigDecimal>,
        payload: &CreatePostAudit,
    ) -> Result<InvestmentPostAudit, AppError> {
        let row = sqlx::query_as::<_, InvestmentPostAudit>(
            r#"
            INSERT INTO capital.investment_post_audits (
                case_id, audit_date, promised_npv, promised_irr,
                actual_npv, actual_irr, actual_payback,
                variance_notes, lessons_learned, performance_link, audited_by
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *
            "#,
        )
        .bind(case_id)
        .bind(payload.audit_date)
        .bind(&promised_npv)
        .bind(&promised_irr)
        .bind(&payload.actual_npv)
        .bind(&payload.actual_irr)
        .bind(&payload.actual_payback)
        .bind(&payload.variance_notes)
        .bind(&payload.lessons_learned)
        .bind(&payload.performance_link)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        sqlx::query(
            r#"
            UPDATE capital.investment_cases SET
                actual_npv = $2, actual_irr = $3,
                audit_completed_at = $4, lessons_learned = $5,
                stage = 'POST_AUDIT', stage_changed_at = now(), updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(case_id)
        .bind(&payload.actual_npv)
        .bind(&payload.actual_irr)
        .bind(payload.audit_date)
        .bind(&payload.lessons_learned)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    async fn portfolio_board(
        &self,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<Vec<PortfolioBoardRow>, AppError> {
        sqlx::query_as::<_, PortfolioBoardRow>(
            r#"
            SELECT
                c.id AS case_id, c.case_code, c.title, c.investment_type, c.stage,
                c.risk_rating, c.strategic_score, c.npv, c.irr, c.payback_years,
                p.program_code, p.name AS program_name,
                c.project_id, pr.approved_budget, pr.spent_to_date,
                CASE WHEN pr.approved_budget > 0
                     THEN ROUND(pr.spent_to_date / pr.approved_budget, 4)
                     ELSE NULL END AS budget_burn_pct
            FROM capital.investment_cases c
            LEFT JOIN capital.investment_programs p ON p.id = c.program_id
            LEFT JOIN capital.capital_projects pr ON pr.id = c.project_id
            WHERE c.company_id = $1
              AND c.stage NOT IN ('REJECTED','CANCELLED')
            ORDER BY c.created_at DESC
            "#,
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }
}

