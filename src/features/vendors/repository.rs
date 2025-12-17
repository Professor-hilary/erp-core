use crate::{
    infrastructure::errors::AppError,
    models::vendor::{ApplyPayment, Bill, CreateBill, CreateVendor, Payment, PostBill, Vendor},
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
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

    async fn create_bill(&self, pool: &PgPool, payload: &CreateBill) -> Result<Bill, AppError>;

    async fn post_bill(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostBill,
    ) -> Result<i64, AppError>;

    async fn list_vendor_bills(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Bill>, AppError>;

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<Payment, AppError>;
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

    async fn create_bill(&self, pool: &PgPool, payload: &CreateBill) -> Result<Bill, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;
        let mut total_cost: BigDecimal = Default::default();
        let mut tax_amount: BigDecimal = Default::default();

        let bill: Bill = sqlx::query_as::<_, Bill>(
            r#"
            INSERT INTO payables.bills (
                bill_number,
                vendor_uuid,
                bill_date,
                due_date,
                reference,
                total_amount,
                tax_amount,
                currency
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(&payload.bill_number)
        .bind(payload.vendor_uuid)
        .bind(payload.bill_date)
        .bind(payload.due_date)
        .bind(&payload.reference)
        .bind(&payload.total_amount)
        .bind(&payload.tax_amount)
        .bind(&payload.currency)
        .fetch_one(&mut *tx)
        .await?;

        for item in &payload.items {
            let total_before_tax: BigDecimal = &item.quantity * &item.unit_price;
            let gross_tax: BigDecimal = &total_before_tax * (&item.tax_rate / 100);
            let total_after_tax: BigDecimal = &total_before_tax + &gross_tax;

            sqlx::query(
                r#"
                INSERT INTO payables.bill_items (
                    bill_uuid,
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
            .bind(bill.uuid)
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

        // Update bill with total and tax computed from items' meta
        sqlx::query(
            r#"
                UPDATE payables.bills SET total_amount=$1, tax_amount=$2
                    WHERE uuid=$3 RETURNING *
            "#,
        )
        .bind(&total_cost)
        .bind(&tax_amount)
        .bind(bill.uuid)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(bill)
    }

    async fn post_bill(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostBill,
    ) -> Result<i64, AppError> {
        sqlx::query(r#"SELECT payables.post_bill($1, $2, $3, $4)"#)
            .bind(payload.bill_serial_id)
            .bind(user_id)
            .bind(&payload.inventory_account)
            .bind(&payload.payables_account)
            .execute(pool)
            .await?;

        Ok(payload.bill_serial_id)
    }

    async fn list_vendor_bills(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Bill>, AppError> {
        let bills = sqlx::query_as::<_, Bill>(
            r#"
            SELECT *
            FROM payables.bills
            WHERE vendor_uuid = $1
            ORDER BY bill_date DESC
            "#,
        )
        .bind(vendor_uuid)
        .fetch_all(pool)
        .await?;

        Ok(bills)
    }

    async fn apply_payment(&self, pool: &PgPool, cmd: ApplyPayment) -> Result<Payment, AppError> {
        let mut tx = pool.begin().await?;

        let payment = sqlx::query_as::<_, Payment>(r#"SELECT payables.apply_payment($1, $2, $3)"#)
            .bind(cmd.payment_serial_id)
            .bind(cmd.bill_serial_id)
            .bind(&cmd.amount)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(AppError::NotFound("Failed to apply payment".into()))?;

        tx.commit().await?;

        Ok(payment)
    }
}
