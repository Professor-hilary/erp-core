use crate::infrastructure::errors::AppError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn get_balancesheet(&self, pool: &PgPool, user_id: Uuid) -> Result<(), AppError>;
    async fn get_income(&self, pool: &PgPool, user_id: Uuid) -> Result<(), AppError>;
    async fn get_cf_direct(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid)
    -> Result<(), AppError>;
    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
    async fn get_aging_ar(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn get_aging_ap(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn get_sales_report(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
    async fn get_purchase_report(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
}

pub struct PostgresReportRepo;

impl PostgresReportRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ReportRepository for PostgresReportRepo {
    async fn get_balancesheet(&self, pool: &PgPool, _user_id: Uuid) -> Result<(), AppError> {
        let _ = sqlx::query(
            r#"
            INSERT INTO receivables.customers (name, code, email, phone, billing_address, credit_limit, current_balance)
            VALUES ($1,$2,$3,$4,$5, COALESCE($6,0),0)
            RETURNING *
            "#,
        )
        .fetch_one(pool)
        .await?;
        Ok(())
    }

    async fn get_income(&self, pool: &PgPool, _user_id: Uuid) -> Result<(), AppError> {
        let _ = sqlx::query("SELECT * FROM receivables.customers")
            .fetch_all(pool)
            .await?;
        Ok(())
    }

    async fn get_cf_direct(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query("SELECT * FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Report not found".into()))?;
        Ok(())
    }
    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query("SELECT * FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Report not found".into()))?;
        Ok(())
    }

    async fn get_aging_ar(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query(
            r#"
            UPDATE receivables.customers
            SET name=$1, email=$2, phone=$3, billing_address=$4,
                credit_limit=COALESCE($5, credit_limit)
            WHERE uuid=$6
            RETURNING *
            "#,
        )
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(())
    }

    async fn get_aging_ap(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query("DELETE FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }
    async fn get_sales_report(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query("DELETE FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }
    async fn get_purchase_report(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let _ = sqlx::query("DELETE FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }
}
