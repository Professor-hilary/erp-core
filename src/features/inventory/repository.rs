// src/features/inventory/repository.rs
use crate::{
    infrastructure::errors::AppError,
    models::{
        inventory_movement::{PostPurchase, PostSale},
        item::{CreateItem, Item},
    },
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn create_item(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError>;
    async fn list_items(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Item>, AppError>;
    async fn get_item(&self, pool: &PgPool, id: i64, user_id: Uuid) -> Result<Item, AppError>;
    async fn update_item(
        &self,
        pool: &PgPool,
        id: i64,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError>;
    async fn delete_item(&self, pool: &PgPool, id: i64, user_id: Uuid) -> Result<(), AppError>;

    async fn post_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<(), AppError>;
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
        let item = sqlx::query_as::<_, Item>(
            r#"
            INSERT INTO inventory.items (
                sku, name, category_id, description, unit, cost_price, selling_price,
                track_quantity, reorder_level, asset_account, cogs_account, income_account
            )
            VALUES (
                $1, $2, $3, $4, COALESCE($5, 'pcs'), COALESCE($6, 0), COALESCE($7, 0),
                COALESCE($8, TRUE), COALESCE($9, 0), COALESCE($10, '1.3.1'), COALESCE($11, '5.2.1'),
                COALESCE($12, '4.1.1')) RETURNING *
            "#
        )
        .bind(&payload.sku)
        .bind(&payload.name)
        .bind(payload.category_id)
        .bind(&payload.description)
        .bind(&payload.unit)
        .bind(payload.cost_price.as_ref())
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

    async fn list_items(&self, pool: &PgPool, _user_id: Uuid) -> Result<Vec<Item>, AppError> {
        let rows = sqlx::query_as::<_, Item>("SELECT * FROM inventory.items")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }

    async fn get_item(&self, pool: &PgPool, id: i64, _user_id: Uuid) -> Result<Item, AppError> {
        let item = sqlx::query_as::<_, Item>("SELECT * FROM inventory.items WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound("Item not found".into()))?;
        Ok(item)
    }

    async fn update_item(
        &self,
        pool: &PgPool,
        id: i64,
        _user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError> {
        let item = sqlx::query_as::<_, Item>(
            r#"
            UPDATE inventory.items
            SET sku=$1, name=$2, category_id=$3, description=$4, unit=COALESCE($5, unit),
                cost_price=COALESCE($6, cost_price), selling_price=COALESCE($7, selling_price),
                track_quantity=COALESCE($8, track_quantity), reorder_level=COALESCE($9, reorder_level),
                asset_account=COALESCE($10, asset_account), cogs_account=COALESCE($11, cogs_account),
                income_account=COALESCE($12, income_account), updated_at=now()
            WHERE id=$13
            RETURNING *
            "#
        )
        .bind(&payload.sku)
        .bind(&payload.name)
        .bind(payload.category_id)
        .bind(&payload.description)
        .bind(&payload.unit)
        .bind(payload.cost_price.as_ref())
        .bind(payload.selling_price.as_ref())
        .bind(payload.track_quantity)
        .bind(payload.reorder_level.as_ref())
        .bind(&payload.asset_account)
        .bind(&payload.cogs_account)
        .bind(&payload.income_account)
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(item)
    }

    async fn delete_item(&self, pool: &PgPool, id: i64, _user_id: Uuid) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM inventory.items WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Item not found".into()))
        } else {
            Ok(())
        }
    }

    async fn post_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<(), AppError> {
        sqlx::query("SELECT inventory.post_purchase($1, $2, $3, $4, $5, $6)")
            .bind(payload.item_id)
            .bind(&payload.quantity)
            .bind(&payload.unit_cost)
            .bind(&payload.reference_type)
            .bind(payload.reference_id)
            .bind(user_id) // assuming user_id as BIGINT
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn post_sale(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostSale,
    ) -> Result<(), AppError> {
        sqlx::query("SELECT inventory.post_sale($1, $2, $3, $4, $5, $6)")
            .bind(payload.item_id)
            .bind(&payload.quantity)
            .bind(&payload.unit_cost)
            .bind(&payload.reference_type)
            .bind(payload.reference_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
