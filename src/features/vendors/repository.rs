use crate::{
    infrastructure::errors::AppError,
    models::vendor::{CreateVendor, Vendor},
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait VendorRepository: Send + Sync {
    async fn create(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError>;
    async fn list(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Vendor>, AppError>;
    async fn get(&self, pool: &PgPool, id: i64, user_id: Uuid) -> Result<Vendor, AppError>;
    async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError>;
    async fn delete(&self, pool: &PgPool, id: i64, user_id: Uuid) -> Result<(), AppError>;
}

pub struct PostgresVendorRepo;

impl PostgresVendorRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VendorRepository for PostgresVendorRepo {
    async fn create(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError> {
        let cust = sqlx::query_as::<_, Vendor>(
            r#"
            INSERT INTO payables.vendors (name, email, phone, billing_address, credit_limit, current_balance)
            VALUES ($1,$2,$3,$4,COALESCE($5,0),0)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.billing_address)
        .bind(payload.credit_limit.as_ref())
        .fetch_one(pool)
        .await?;
        Ok(cust)
    }

    async fn list(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Vendor>, AppError> {
        let rows = sqlx::query_as::<_, Vendor>("SELECT * FROM payables.vendors")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn get(&self, pool: &PgPool, id: i64, _user_id: Uuid) -> Result<Vendor, AppError> {
        let cust = sqlx::query_as::<_, Vendor>("SELECT * FROM payables.vendors WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Vendor not found".into()))?;
        Ok(cust)
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        _user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError> {
        let cust = sqlx::query_as::<_, Vendor>(
            r#"
            UPDATE payables.vendors
            SET name=$1, email=$2, phone=$3, billing_address=$4,
                credit_limit=COALESCE($5, credit_limit)
            WHERE id=$6
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.billing_address)
        .bind(payload.credit_limit.as_ref())
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(cust)
    }

    async fn delete(&self, pool: &PgPool, id: i64, _user_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM payables.vendors WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Vendor not found".into()))
        } else {
            Ok(())
        }
    }
}
