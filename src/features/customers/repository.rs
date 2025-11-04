use crate::{errors::AppError, models::customers::{CreateCustomer, Customer}};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError>;
    async fn list(&self, user_id: Uuid) -> Result<Vec<Customer>, AppError>;
    async fn get(&self, id: i64, user_id: Uuid) -> Result<Customer, AppError>;
    async fn update(&self, id: i64, user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError>;
    async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError>;
}

pub struct PostgresCustomerRepo { pool: PgPool }

impl PostgresCustomerRepo {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl CustomerRepository for PostgresCustomerRepo {
    async fn create(&self, _user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError> {
        let cust = sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO receivables.customers (name, email, phone, billing_address, credit_limit, current_balance)
            VALUES ($1,$2,$3,$4,COALESCE($5,0),0)
            RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.billing_address)
        .bind(payload.credit_limit.as_ref())
        .fetch_one(&self.pool)
        .await?;
        Ok(cust)
    }

    async fn list(&self, _user_id: Uuid) -> Result<Vec<Customer>, AppError> {
        let rows = sqlx::query_as::<_, Customer>("SELECT * FROM receivables.customers")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }

    async fn get(&self, id: i64, _user_id: Uuid) -> Result<Customer, AppError> {
        let cust = sqlx::query_as::<_, Customer>("SELECT * FROM receivables.customers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("Customer not found".into()))?;
        Ok(cust)
    }

    async fn update(&self, id: i64, _user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError> {
        let cust = sqlx::query_as::<_, Customer>(
            r#"
            UPDATE receivables.customers
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
        .fetch_one(&self.pool)
        .await?;
        Ok(cust)
    }

    async fn delete(&self, id: i64, _user_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM receivables.customers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Customer not found".into()))
        } else {
            Ok(())
        }
    }
}