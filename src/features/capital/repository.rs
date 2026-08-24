use crate::{
    interface::api::errors::AppError,
    models::capital::{
        CapitalFacility, CreateCapitalFacility, CreateDrawdown, CreateRepayment,
        DebtDrawdown, DebtRepayment,
    },
};

use async_trait::async_trait;
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

#[async_trait]
pub trait CapitalRepository: Send + Sync {
    // Debt Facilities
    async fn create_facility(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCapitalFacility,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<CapitalFacility, AppError>;

    async fn get_facility(
        &self,
        pool: &PgPool,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError>;

    async fn get_all_facilities(
        &self,
        pool: &PgPool,
    ) -> Result<Vec<CapitalFacility>, AppError>;

    // Drawdowns
    async fn create_drawdown(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateDrawdown,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<DebtDrawdown, AppError>;

    async fn get_drawdowns(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError>;

    // Repayments
    async fn create_repayment(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateRepayment,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<DebtRepayment, AppError>;

    async fn get_repayments(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError>;
}


pub struct PostgresCapitalRepo;

impl PostgresCapitalRepo {
    pub fn new() -> Self {
        Self
    }
}


#[async_trait]
impl CapitalRepository for PostgresCapitalRepo {
    async fn create_facility(
        &self,
        _pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateCapitalFacility,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<CapitalFacility, AppError> {
        let facility = sqlx::query_as::<_, CapitalFacility>(
            r#"
            INSERT INTO capital.debt_facilities (
                lender_id,
                facility_name,
                facility_type,
                currency_id,
                approved_limit,
                interest_rate,
                interest_type,
                effective_date,
                maturity_date,
                collateral_required
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10
            )
            RETURNING *
            "#,
        )
        .bind(payload.lender_id)
        .bind(&payload.facility_name)
        .bind(&payload.facility_type)
        .bind(payload.currency_id)
        .bind(&payload.approved_limit)
        .bind(&payload.interest_rate)
        .bind(&payload.interest_type)
        .bind(payload.effective_date)
        .bind(payload.maturity_date)
        .bind(payload.collateral_required)
        .fetch_one(&mut **tx)
        .await?;

        Ok(facility)
    }

    async fn get_facility(
        &self,
        pool: &PgPool,
        id: Uuid,
    ) -> Result<CapitalFacility, AppError> {
        sqlx::query_as::<_, CapitalFacility>(
            r#"
            SELECT *
            FROM capital.debt_facilities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Capital facility not found".into()))
    }

    async fn get_all_facilities(
        &self,
        pool: &PgPool,
    ) -> Result<Vec<CapitalFacility>, AppError> {
        let facilities = sqlx::query_as::<_, CapitalFacility>(
            r#"
            SELECT *
            FROM capital.debt_facilities
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(facilities)
    }

    async fn create_drawdown(
        &self,
        _pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateDrawdown,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<DebtDrawdown, AppError> {
        let drawdown = sqlx::query_as::<_, DebtDrawdown>(
            r#"
            INSERT INTO capital.debt_drawdowns (
                facility_id,
                drawdown_date,
                amount,
                reference
            )
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.drawdown_date)
        .bind(&payload.amount)
        .bind(&payload.reference)
        .fetch_one(&mut **tx)
        .await?;

        Ok(drawdown)
    }

    async fn get_drawdowns(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtDrawdown>, AppError> {
        let drawdowns = sqlx::query_as::<_, DebtDrawdown>(
            r#"
            SELECT *
            FROM capital.debt_drawdowns
            WHERE facility_id = $1
            ORDER BY drawdown_date DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(drawdowns)
    }

    async fn create_repayment(
        &self,
        _pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateRepayment,
        tx: &mut Transaction<'_, sqlx::Postgres>,
    ) -> Result<DebtRepayment, AppError> {
        let repayment = sqlx::query_as::<_, DebtRepayment>(
            r#"
            INSERT INTO capital.debt_repayments (
                facility_id,
                repayment_date,
                principal_amount,
                interest_amount
            )
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(payload.facility_id)
        .bind(payload.repayment_date)
        .bind(&payload.principal_amount)
        .bind(&payload.interest_amount)
        .fetch_one(&mut **tx)
        .await?;

        Ok(repayment)
    }

    async fn get_repayments(
        &self,
        pool: &PgPool,
        facility_id: Uuid,
    ) -> Result<Vec<DebtRepayment>, AppError> {
        let repayments = sqlx::query_as::<_, DebtRepayment>(
            r#"
            SELECT *
            FROM capital.debt_repayments
            WHERE facility_id = $1
            ORDER BY repayment_date DESC
            "#,
        )
        .bind(facility_id)
        .fetch_all(pool)
        .await?;

        Ok(repayments)
    }
}