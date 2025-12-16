use crate::{
    infrastructure::errors::AppError,
    models::customers::{
        ApplyPayment, CreateCustomer, CreateInvoice, Customer, Invoice, PostInvoice,
    },
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

    async fn create_invoice(
        &self,
        pool: &PgPool,
        payload: &CreateInvoice,
    ) -> Result<Invoice, AppError>;

    async fn post_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostInvoice,
    ) -> Result<(), AppError>;

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Invoice>, AppError>;

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<(), AppError>;
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

    async fn create_invoice(
        &self,
        pool: &PgPool,
        payload: &CreateInvoice,
    ) -> Result<Invoice, AppError> {
        let mut tx = pool.begin().await?;

        let invoice: Invoice = sqlx::query_as::<_, Invoice>(
            r#"
            INSERT INTO receivables.invoices (
                invoice_number,
                customer_uuid,
                issue_date,
                due_date,
                total_amount,
                tax_amount,
                balance_due,
                currency
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$6,$8)
            RETURNING *
            "#,
        )
        .bind(&payload.invoice_number)
        .bind(payload.customer_uuid)
        .bind(payload.issue_date)
        .bind(payload.due_date)
        .bind(&payload.total_amount)
        .bind(&payload.tax_amount)
        .bind(&payload.total_amount + &payload.tax_amount)
        .bind(&payload.currency)
        .fetch_one(&mut *tx)
        .await?;

        for item in &payload.items {
            sqlx::query(
                r#"
                INSERT INTO receivables.invoice_items (
                    invoice_uuid,
                    item_code,
                    description,
                    quantity,
                    unit_price,
                    tax_rate,
                    total
                )
                VALUES ($1,$2,$3,$4,$5,$6,$7)
                "#,
            )
            .bind(invoice.uuid)
            .bind(&item.item_code)
            .bind(&item.description)
            .bind(&item.quantity)
            .bind(&item.unit_price)
            .bind(&item.tax_rate)
            .bind(&item.total)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(invoice)
    }

    async fn post_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostInvoice,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            SELECT payables.post_invoice($1, $2, $3, $4)
            "#,
        )
        .bind(payload.invoice_serial_id)
        .bind(user_id)
        .bind(&payload.receivable_code)
        .bind(&payload.revenue_code)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
    ) -> Result<Vec<Invoice>, AppError> {
        let invoices = sqlx::query_as::<_, Invoice>(
            r#"
            SELECT *
            FROM receivables.invoices
            WHERE customer_uuid = $1
            ORDER BY issue_date DESC
            "#,
        )
        .bind(customer_uuid)
        .fetch_all(pool)
        .await?;

        Ok(invoices)
    }

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<(), AppError> {
        let mut tx = pool.begin().await?;

        sqlx::query(r#"SELECT receivables.apply_payment($1, $2, $3)"#)
            .bind(cmd.payment_serial_id)
            .bind(cmd.invoice_serial_id)
            .bind(&cmd.amount)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
