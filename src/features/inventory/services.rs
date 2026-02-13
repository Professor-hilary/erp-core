// src/features/inventory/service.rs
use crate::{
    features::inventory::repository::InventoryRepository,
    interface::api::errors::AppError,
    models::{inventory::{
        CreateItem, CreateItemCategory, CreateWarehouse, Item, ItemCategory,
        PostSale, Warehouse,
    }, vendor::CreateBill, },
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct InventoryService<R: InventoryRepository> {
    repo: R,
}

impl<R: InventoryRepository> InventoryService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_item(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError> {
        self.repo.create_item(tenant_pool, user_id, payload).await
    }

    pub async fn create_item_category(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError> {
        self.repo
            .create_item_category(tenant_pool, user_id, payload)
            .await
    }

    pub async fn create_warehouse(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError> {
        self.repo
            .create_warehouse(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_items(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Item>, AppError> {
        self.repo.list_items(tenant_pool, user_id).await
    }

    pub async fn list_item_categories(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<ItemCategory>, AppError> {
        self.repo.list_item_categories(tenant_pool, user_id).await
    }

    pub async fn list_warehouse(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Warehouse>, AppError> {
        self.repo.list_warehouse(tenant_pool, user_id).await
    }

    pub async fn get_item(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Item, AppError> {
        self.repo.get_item(tenant_pool, uuid, user_id).await
    }

    pub async fn get_item_category(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<ItemCategory, AppError> {
        self.repo
            .get_item_category(tenant_pool, uuid, user_id)
            .await
    }

    pub async fn get_warehouse(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Warehouse, AppError> {
        self.repo.get_warehouse(tenant_pool, uuid, user_id).await
    }

    pub async fn update_item(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateItem,
    ) -> Result<Item, AppError> {
        self.repo
            .update_item(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn update_item_category(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateItemCategory,
    ) -> Result<ItemCategory, AppError> {
        self.repo
            .update_item_category(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn update_warehouse(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateWarehouse,
    ) -> Result<Warehouse, AppError> {
        self.repo
            .update_warehouse(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn delete_item(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_item(tenant_pool, uuid, user_id).await
    }

    pub async fn delete_item_category(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_item_category(tenant_pool, uuid, user_id)
            .await
    }

    pub async fn delete_warehouse(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_warehouse(tenant_pool, uuid, user_id).await
    }

    pub async fn post_purchase(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateBill,
    ) -> Result<crate::models::vendor::Bill, AppError> {
        self.repo.cash_purchase(tenant_pool, user_id, payload).await
    }

    pub async fn post_sale(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &PostSale,
    ) -> Result<(), AppError> {
        self.repo.post_sale(tenant_pool, user_id, payload).await
    }
}
