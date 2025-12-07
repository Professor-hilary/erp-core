// src/features/inventory/service.rs
use crate::{
    features::inventory::repository::InventoryRepository,
    infrastructure::errors::AppError,
    models::item::{CreateItem, Item, PostPurchase, PostSale},
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

    pub async fn list_items(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Item>, AppError> {
        self.repo.list_items(tenant_pool, user_id).await
    }

    pub async fn get_item(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Item, AppError> {
        self.repo.get_item(tenant_pool, uuid, user_id).await
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

    pub async fn delete_item(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_item(tenant_pool, uuid, user_id).await
    }

    pub async fn post_purchase(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<(), AppError> {
        self.repo.post_purchase(tenant_pool, user_id, payload).await
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
