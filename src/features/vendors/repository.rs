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
    async fn get(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<Vendor, AppError>;
    async fn update(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError>;
    async fn delete(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;
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
        let vendor: Vendor = sqlx::query_as::<_, Vendor>(
            r#"
            INSERT INTO payables.vendors (code, name, email, phone, address, credit_limit, current_balance)
            VALUES ($1,$2,$3,$4,$5,COALESCE($6,0),0)
            RETURNING *
            "#,
        )
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.address)
        .bind(payload.credit_limit.as_ref())
        .fetch_one(pool)
        .await?;
        Ok(vendor)
    }

    async fn list(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Vendor>, AppError> {
        let vendors: Vec<Vendor> = sqlx::query_as::<_, Vendor>("SELECT * FROM payables.vendors")
            .fetch_all(pool)
            .await?;
        Ok(vendors)
    }

    async fn get(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<Vendor, AppError> {
        let vendor: Vendor =
            sqlx::query_as::<_, Vendor>("SELECT * FROM payables.vendors WHERE uuid = $1")
                .bind(uuid)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Vendor not found".into()))?;
        Ok(vendor)
    }

    async fn update(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError> {
        let vendor: Vendor = sqlx::query_as::<_, Vendor>(
            r#"
                UPDATE payables.vendors
                SET name=$1, email=$2, phone=$3, address=$4,
                    credit_limit=COALESCE($5, credit_limit)
                WHERE uuid=$6 RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.address)
        .bind(payload.credit_limit.as_ref())
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(vendor)
    }

    async fn delete(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<(), AppError> {
        let res: sqlx::postgres::PgQueryResult =
            sqlx::query("DELETE FROM payables.vendors WHERE uuid = $1")
                .bind(uuid)
                .execute(pool)
                .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Vendor not found".into()))
        } else {
            Ok(())
        }
    }
}
