// src/features/inventory/repository.rs
use crate::{
    interface::api::errors::AppError,
    models::{
        inventory::{
            CreateItem, CreateItemCategory, CreateWarehouse, Item, ItemCategory, PostSale,
            Warehouse,
        },
        vendor::{Purchase, CreatePurchase, PostPurchase},
    },
};
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    // Create repo methods
    async fn create_item(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError>;
    async fn create_item_category(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError>;
    async fn create_warehouse(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError>;

    // List repo methods
    async fn list_items(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Item>, AppError>;
    async fn list_item_categories(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<ItemCategory>, AppError>;
    async fn list_warehouse(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<Warehouse>, AppError>;

    // Get repo methods
    async fn get_item(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<Item, AppError>;
    async fn get_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<ItemCategory, AppError>;
    async fn get_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<Warehouse, AppError>;

    // Update repo methods
    async fn update_item(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError>;
    async fn update_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError>;
    async fn update_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError>;

    // Delete repo methods
    async fn delete_item(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;
    async fn delete_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
    async fn delete_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // Create purchase order
    async fn cash_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreatePurchase,
    ) -> Result<Purchase, AppError>;

    // Proceed to procure
    async fn post_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<i64, AppError>;

    async fn post_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostSale,
    ) -> Result<(), AppError>;
}

pub struct PostgresInventoryRepo;

impl PostgresInventoryRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InventoryRepository for PostgresInventoryRepo {
    async fn create_item(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError> {
        let item: Item = sqlx::query_as::<_, Item>(
            r#"
                INSERT INTO inventory.items (
                    sku, name, category_uuid, warehouse_serial, description, unit, selling_price,
                    track_quantity, reorder_level, asset_account, cogs_account,
                    income_account
                )
                VALUES (
                    $1, $2, $3, $4, COALESCE($5, 'pcs'), COALESCE($6, 0),
                    COALESCE($7, true), COALESCE($8, 0), $9, $10, $11
                ) RETURNING *
            "#,
        )
        .bind(&payload.sku)
        .bind(&payload.name)
        .bind(payload.category_uuid)
        .bind(&payload.description)
        .bind(&payload.warehouse_serial)
        .bind(&payload.unit)
        .bind(payload.selling_price.as_ref())
        .bind(payload.track_quantity)
        .bind(payload.reorder_level.as_ref())
        .bind(&payload.asset_account)
        .bind(&payload.cogs_account)
        .bind(&payload.income_account)
        .fetch_one(pool)
        .await?;
        Ok(item)
    }

    async fn create_item_category(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError> {
        let category: ItemCategory = sqlx::query_as::<_, ItemCategory>(
            r#"
            INSERT INTO inventory.item_categories (name, description, code)
            VALUES ($1, $2, $3) RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.code)
        .fetch_one(pool)
        .await?;
        Ok(category)
    }

    async fn create_warehouse(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError> {
        let category: Warehouse = sqlx::query_as::<_, Warehouse>(
            r#"
            INSERT INTO inventory.warehouses (name, description, code, location)
            VALUES ($1, $2, $3, $4) RETURNING *
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.code)
        .bind(&payload.location)
        .fetch_one(pool)
        .await?;
        Ok(category)
    }

    async fn list_items(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Item>, AppError> {
        let items: Vec<Item> = sqlx::query_as::<_, Item>("SELECT * FROM inventory.items")
            .fetch_all(pool)
            .await?;
        Ok(items)
    }

    async fn list_item_categories(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<ItemCategory>, AppError> {
        let categories: Vec<ItemCategory> =
            sqlx::query_as::<_, ItemCategory>("SELECT * FROM inventory.item_categories")
                .fetch_all(pool)
                .await?;
        Ok(categories)
    }

    async fn list_warehouse(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<Vec<Warehouse>, AppError> {
        let warehouses: Vec<Warehouse> =
            sqlx::query_as::<_, Warehouse>("SELECT * FROM inventory.warehouses")
                .fetch_all(pool)
                .await?;
        Ok(warehouses)
    }

    async fn get_item(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<Item, AppError> {
        let item: Item = sqlx::query_as::<_, Item>("SELECT * FROM inventory.items WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Item not found".into()))?;
        Ok(item)
    }

    async fn get_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<ItemCategory, AppError> {
        let item: ItemCategory = sqlx::query_as::<_, ItemCategory>(
            "SELECT * FROM inventory.item_categories WHERE uuid = $1",
        )
        .bind(uuid)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Item category not found".into()))?;
        Ok(item)
    }

    async fn get_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<Warehouse, AppError> {
        let warehouse: Warehouse =
            sqlx::query_as::<_, Warehouse>("SELECT * FROM inventory.warehouses WHERE uuid = $1")
                .bind(uuid)
                .fetch_optional(pool)
                .await?
                .ok_or(AppError::NotFound("Warehouse not found".into()))?;
        Ok(warehouse)
    }

    async fn update_item(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError> {
        let item: Item = sqlx::query_as::<_, Item>(
            r#"
                UPDATE inventory.items
                SET
                    sku=COALESCE($1, sku),
                    name=COALESCE($2 name),
                    category_uuid=COALESCE($3, category_uuid),
                    description=COALESCE($4, description),
                    unit=COALESCE($5, unit),
                    selling_price=COALESCE($7, selling_price),
                    track_quantity=COALESCE($8, track_quantity),
                    reorder_level=COALESCE($9, reorder_level),
                    asset_account=COALESCE($10, asset_account),
                    cogs_account=COALESCE($11, cogs_account),
                    income_account=COALESCE($12, income_account),
                    warehouse_serial=COALESCE($13, warehouse_serial),
                    updated_at=now()
                WHERE uuid=$13
                RETURNING *
            "#,
        )
        .bind(&payload.sku)
        .bind(&payload.name)
        .bind(payload.category_uuid)
        .bind(&payload.description)
        .bind(&payload.unit)
        .bind(payload.selling_price.as_ref())
        .bind(payload.track_quantity)
        .bind(payload.reorder_level.as_ref())
        .bind(&payload.asset_account)
        .bind(&payload.cogs_account)
        .bind(&payload.income_account)
        .bind(&payload.warehouse_serial)
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(item)
    }

    async fn update_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError> {
        let category: ItemCategory = sqlx::query_as::<_, ItemCategory>(
            r#"UPDATE
                inventory.item_categories SET code=COALESCE($1, code), name=COALESCE($2, name),
                description=COALESCE($3, description), updated_at=now()
            WHERE uuid=$4 RETURNING *"#,
        )
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(category)
    }

    async fn update_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError> {
        let warehouse: Warehouse = sqlx::query_as::<_, Warehouse>(
            r#"UPDATE
                inventory.warehouses SET code=COALESCE($1, code), name=COALESCE($2, name),
                description=COALESCE($3, description), location=COALESCE($4, location)
            WHERE uuid=$5 RETURNING *"#,
        )
        .bind(&payload.code)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.location)
        .bind(uuid)
        .fetch_one(pool)
        .await?;
        Ok(warehouse)
    }

    async fn delete_item(&self, pool: &PgPool, uuid: Uuid, _user_id: Uuid) -> Result<(), AppError> {
        let res: PgQueryResult = sqlx::query("DELETE FROM inventory.items WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Item not found".into()))
        } else {
            Ok(())
        }
    }

    async fn delete_item_category(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let res: PgQueryResult =
            sqlx::query("DELETE FROM inventory.item_categories WHERE uuid = $1")
                .bind(uuid)
                .execute(pool)
                .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Item category not found".into()))
        } else {
            Ok(())
        }
    }

    async fn delete_warehouse(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let res: PgQueryResult = sqlx::query("DELETE FROM inventory.warehouses WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Warehouse not found".into()))
        } else {
            Ok(())
        }
    }

    async fn cash_purchase(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        payload: &CreatePurchase,
    ) -> Result<Purchase, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;
        let mut total_cost: BigDecimal = Default::default();
        let mut tax_amount: BigDecimal = Default::default();

        let bill: Purchase = sqlx::query_as::<_, Purchase>(
            r#"
            INSERT INTO procurement.purchases (
                bill_number,
                vendor_uuid,
                bill_date,
                due_date,
                reference,
                total_amount,
                tax_amount,
                settlement_type,
                paid_at,
                payment_status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'paid')
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
        .bind(&payload.settlement_type)
        .bind(&payload.paid_at)
        .fetch_one(&mut *tx)
        .await?;

        for item in &payload.items {
            let total_before_tax: BigDecimal = &item.quantity * &item.unit_price;
            let gross_tax: BigDecimal = &total_before_tax * (&item.tax_rate / 100);
            let total_after_tax: BigDecimal = &total_before_tax + &gross_tax;

            sqlx::query(
                r#"
                INSERT INTO procurement.purchase_items (
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
                UPDATE procurement.purchases SET total_amount=$1, tax_amount=$2
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

    async fn post_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<i64, AppError> {
        sqlx::query(r#"
                SELECT procurement.procure_stock($1, $2, $3, null, $4)
            "#)
            .bind(payload.bill_serial_id)
            .bind(user_id)
            .bind(&payload.vat_tax_account)
            .bind(&payload.cash_account)
            .execute(pool)
            .await?;

        Ok(payload.bill_serial_id)
    }

    async fn post_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostSale,
    ) -> Result<(), AppError> {
        sqlx::query("SELECT inventory.post_sale($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(payload.item_serial_id)
            .bind(payload.warehouse_serial_id)
            .bind(&payload.quantity)
            .bind(&payload.unit_cost)
            .bind(&payload.reference_type)
            .bind(payload.reference_serial_id)
            .bind(user_id)
            .bind(&payload.cash_account_code)
            .execute(pool)
            .await?;
        Ok(())
    }
}
