use crate::{
    interface::api::errors::AppError,
    models::customers::{
        ApplyPayment, CreateCustomer, CreateInvoice, Customer, Invoice, Payment, PostInvoice,
    },
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
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
    async fn delete(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<u64, AppError>;

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
    ) -> Result<Invoice, AppError>;

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Invoice>, AppError>;

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<Payment, AppError>;
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
            INSERT INTO sales.customers (name, code, email, phone, billing_address, credit_limit, current_balance)
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
            sqlx::query_as::<_, Customer>("SELECT * FROM sales.customers")
                .fetch_all(pool)
                .await?;
        Ok(rows)
    }

    async fn get(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<Customer, AppError> {
        let cust: Customer =
            sqlx::query_as::<_, Customer>("SELECT * FROM sales.customers WHERE uuid = $1")
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
            UPDATE sales.customers
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

    async fn delete(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<u64, AppError> {
        let res: PgQueryResult = sqlx::query("DELETE FROM sales.customers WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Customer not found".into()))
        } else {
            Ok(res.rows_affected())
        }
    }

    async fn create_invoice(
        &self,
        pool: &PgPool,
        payload: &CreateInvoice,
    ) -> Result<Invoice, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;
        let mut total_cost: BigDecimal = Default::default();
        let mut tax_amount: BigDecimal = Default::default();

        let invoice: Invoice = sqlx::query_as::<_, Invoice>(
            r#"
            INSERT INTO sales.turnover (
                invoice_number,
                customer_uuid,
                issue_date,
                due_date,
                total_amount,
                tax_amount,
                balance_due
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(&payload.invoice_number)
        .bind(payload.customer_uuid)
        .bind(payload.issue_date)
        .bind(payload.due_date)
        .bind(&payload.total_amount)
        .bind(&payload.tax_amount)
        .bind(&payload.total_amount)
        // .bind(&payload.currency)
        .fetch_one(&mut *tx)
        .await?;

        for item in &payload.items {
            let total_before_tax: BigDecimal = &item.quantity * &item.unit_price;
            let gross_tax: BigDecimal = &total_before_tax * (&item.tax_rate / 100);
            let total_after_tax: BigDecimal = &total_before_tax + &gross_tax;

            sqlx::query(
                r#"
                INSERT INTO sales.turnover_items (
                    invoice_uuid,
                    stock_item_id,
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
            .bind(&item.stock_item_id)
            .bind(&item.description)
            .bind(&item.quantity)
            .bind(&item.unit_price)
            .bind(&item.tax_rate)
            .bind(&total_after_tax)
            .execute(&mut *tx)
            .await?;

            total_cost += &total_before_tax;
            tax_amount += &gross_tax;
        }

        // Update invoice with total and tax computed from items' meta
        sqlx::query(
            r#"
                UPDATE sales.turnover SET total_amount=$1, tax_amount=$2
                    WHERE uuid=$3 RETURNING *
            "#,
        )
        .bind(&total_cost)
        .bind(&tax_amount)
        .bind(invoice.uuid)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(invoice)
    }

    async fn post_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostInvoice,
    ) -> Result<Invoice, AppError> {
        let invoice = sqlx::query_as::<_, Invoice>(
            r#"
            SELECT sales.post_turnover($1, $2, $3, $4)
            "#,
        )
        .bind(payload.invoice_serial_id)
        .bind(user_id)
        .bind(&payload.receivable_code)
        .bind(&payload.revenue_code)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Invoice posting failed".into()))?;

        Ok(invoice)
    }

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
    ) -> Result<Vec<Invoice>, AppError> {
        let invoices = sqlx::query_as::<_, Invoice>(
            r#"
            SELECT *
            FROM sales.turnover
            WHERE customer_uuid = $1
            ORDER BY issue_date DESC
            "#,
        )
        .bind(customer_uuid)
        .fetch_all(pool)
        .await?;

        Ok(invoices)
    }

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<Payment, AppError> {
        let mut tx = pool.begin().await?;

        let payment =
            sqlx::query_as::<_, Payment>(r#"SELECT sales.apply_payment($1, $2, $3)"#)
                .bind(cmd.payment_serial_id)
                .bind(cmd.invoice_serial_id)
                .bind(&cmd.amount)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(AppError::NotFound("Payment not found".into()))?;

        tx.commit().await?;

        Ok(payment)
    }
}
