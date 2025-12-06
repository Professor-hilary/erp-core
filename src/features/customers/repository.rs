use crate::{
    infrastructure::errors::AppError,
    models::customers::{CreateCustomer, Customer},
};
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError>;
    async fn list(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Customer>, AppError>;
    async fn get(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<Customer, AppError>;
    async fn update(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError>;
    async fn delete(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;
}

pub struct PostgresCustomerRepo;

impl PostgresCustomerRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CustomerRepository for PostgresCustomerRepo {
    async fn create(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError> {
        let cust: Customer = sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO receivables.customers (name, code, email, phone, billing_address, credit_limit, current_balance)
            VALUES ($1,$2,$3,$4,$5, COALESCE($6,0),0)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.code)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.billing_address)
        .bind(payload.credit_limit.as_ref())
        .fetch_one(pool)
        .await?;
        Ok(cust)
    }

    async fn list(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Customer>, AppError> {
        let rows: Vec<Customer> =
            sqlx::query_as::<_, Customer>("SELECT * FROM receivables.customers")
                .fetch_all(pool)
                .await?;
        Ok(rows)
    }

    async fn get(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<Customer, AppError> {
        let cust: Customer =
            sqlx::query_as::<_, Customer>("SELECT * FROM receivables.customers WHERE uuid = $1")
                .bind(uuid)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Customer not found".into()))?;
        Ok(cust)
    }

    async fn update(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError> {
        let cust: Customer = sqlx::query_as::<_, Customer>(
            r#"
            UPDATE receivables.customers
            SET name=$1, email=$2, phone=$3, billing_address=$4,
                credit_limit=COALESCE($5, credit_limit)
            WHERE uuid=$6
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.billing_address)
        .bind(payload.credit_limit.as_ref())
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(cust)
    }

    async fn delete(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<(), AppError> {
        let res: PgQueryResult = sqlx::query("DELETE FROM receivables.customers WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Customer not found".into()))
        } else {
            Ok(())
        }
    }
}
