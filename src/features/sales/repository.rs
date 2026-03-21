use crate::{
    interface::api::errors::AppError,
    models::customers::{
        ApplyPayment, CreateCustomer, CreateTurnover, Customer, Payment, PostTurnover, Turnover,
    },
};
use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use sqlx::{PgPool, Row, postgres::PgQueryResult};
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

    async fn create_sale_invoice(
        &self,
        pool: &PgPool,
        payload: &CreateTurnover,
    ) -> Result<Turnover, AppError>;

    async fn post_credit_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError>;

    async fn post_cash_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError>;

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Turnover>, AppError>;

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
        let rows: Vec<Customer> = sqlx::query_as::<_, Customer>("SELECT * FROM sales.customers")
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

    /// Handles both cash and credit sales
    async fn create_sale_invoice(
        &self,
        pool: &PgPool,
        payload: &CreateTurnover,
    ) -> Result<Turnover, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        // ---------------------------------------------------------------------------
        // 1. Invoice defaults
        // ---------------------------------------------------------------------------
        let is_cash: bool = payload.settlement_type == "cash";

        let status: Option<&str> = if is_cash { Some("paid") } else { None };
        let paid_at: Option<&str> = if is_cash { Some("now()") } else { None };
        let balance_due = if is_cash {
            BigDecimal::zero()
        } else {
            payload.total_amount.clone().unwrap_or_default()
        };

        // First insert - no total amount and tax
        let invoice: Turnover = sqlx::query_as::<_, Turnover>(
            r#"
            INSERT INTO sales.turnover (
                invoice_number,
                customer_uuid,
                issue_date,
                due_date,
                total_amount,
                tax_amount,
                balance_due,
                settlement_type,
                status,
                paid_at
            )
            VALUES ($1, $2, $3, $4, 0, 0, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&payload.invoice_number)
        .bind(payload.customer_uuid)
        .bind(payload.issue_date)
        .bind(payload.due_date)
        .bind(&balance_due)
        .bind(payload.settlement_type.as_str()) // 'cash' or 'credit'
        .bind(status)
        .bind(paid_at)
        .fetch_one(&mut *tx)
        .await?;

        // ---------------------------------------------------------------------------
        // 2. Line items - identical for both types
        // ---------------------------------------------------------------------------
        let mut subtotal = BigDecimal::zero();
        let mut total_tax = BigDecimal::zero();

        for item in &payload.items {
            let unit_price: BigDecimal = if item.unit_price > BigDecimal::zero() {
                item.unit_price.clone()
            } else {
                // Fetch default selling price from items table
                let row_opt = sqlx::query(
                    r#"SELECT selling_price FROM inventory.items WHERE serial_id = $1"#,
                )
                .bind(item.stock_item_id)
                .fetch_optional(&mut *tx)
                .await?;

                let default_price: BigDecimal = match row_opt {
                    Some(row) => row
                        .try_get::<BigDecimal, _>("selling_price")
                        .map_err(|e: sqlx::Error| AppError::Database(e))?,
                    None => {
                        return Err(AppError::NotFound(format!(
                            "Item {} not found",
                            item.stock_item_id
                        )));
                    }
                };

                if default_price <= BigDecimal::zero() {
                    return Err(AppError::Internal(format!(
                        "Item {} has no valid selling price",
                        item.stock_item_id
                    )));
                }

                default_price
            };

            let amount_ex_tax: BigDecimal = &item.quantity * &unit_price;
            let tax_amount: BigDecimal = &amount_ex_tax * (&item.tax_rate / BigDecimal::from(100));
            let total_after_tax: BigDecimal = &amount_ex_tax + &tax_amount;

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
            .bind(item.stock_item_id)
            .bind(&item.description)
            .bind(&item.quantity)
            .bind(&unit_price)
            .bind(&item.tax_rate)
            .bind(&total_after_tax)
            .execute(&mut *tx)
            .await?;

            subtotal += &amount_ex_tax;
            total_tax += &tax_amount;
        }

        // ---------------------------------------------------------------------------
        // 3. Finalize header with computed values
        // ---------------------------------------------------------------------------
        let final_balance_due = if is_cash {
            BigDecimal::zero()
        } else {
            &subtotal + &total_tax
        };

        // Update invoice with total and tax computed from items' meta
        let updated = sqlx::query_as::<_, Turnover>(
            r#"
            UPDATE sales.turnover
            SET
                total_amount    = $1,
                tax_amount      = $2,
                balance_due     = $3,
                status          = COALESCE($4, status),
                paid_at         = COALESCE($5, paid_at)
            WHERE uuid          = $6
            RETURNING *
            "#,
        )
        .bind(&subtotal)
        .bind(&total_tax)
        .bind(&final_balance_due)
        .bind(status)
        .bind(paid_at)
        .bind(invoice.uuid)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(updated)
    }

    async fn post_credit_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError> {
        let invoice =
            sqlx::query_as::<_, Turnover>(r#"SELECT sales.post_turnover($1, $2, $3, $4)"#)
                .bind(payload.invoice_serial_id)
                .bind(user_id)
                .bind(&payload.output_vat_code)
                .bind(&payload.receivable_code)
                .bind(&payload.revenue_code)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Invoice posting failed".into()))?;

        Ok(invoice)
    }

    async fn post_cash_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError> {
        let sale: Turnover = sqlx::query_as::<_, Turnover>(
            r#"SELECT sales.post_turnover($1, $2, $3, null, $4, $5)"#,
        )
        .bind(payload.invoice_serial_id)
        .bind(user_id)
        .bind(&payload.output_vat_code)
        .bind(&payload.revenue_code)
        .bind(&payload.cash_account_code)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Cash sale posting failed".into()))?;

        Ok(sale)
    }

    async fn list_customer_invoices(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
    ) -> Result<Vec<Turnover>, AppError> {
        let invoices = sqlx::query_as::<_, Turnover>(
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
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let payment: Payment =
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
