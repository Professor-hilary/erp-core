// src/features/inventory/service.rs
use crate::{errors::AppError, features::inventory::repository::InventoryRepository, models::{inventory_movement::{PostPurchase, PostSale}, item::{CreateItem, Item}}};
use uuid::Uuid;

pub struct InventoryService<R: InventoryRepository> {
    repo: R,
}

impl<R: InventoryRepository> InventoryService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_item(&self, user_id: Uuid, payload: &CreateItem) -> Result<Item, AppError> {
        self.repo.create_item(user_id, payload).await
    }

    pub async fn list_items(&self, user_id: Uuid) -> Result<Vec<Item>, AppError> {
        self.repo.list_items(user_id).await
    }

    pub async fn get_item(&self, id: i64, user_id: Uuid) -> Result<Item, AppError> {
        self.repo.get_item(id, user_id).await
    }

    pub async fn update_item(&self, id: i64, user_id: Uuid, payload: &CreateItem) -> Result<Item, AppError> {
        self.repo.update_item(id, user_id, payload).await
    }

    pub async fn delete_item(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete_item(id, user_id).await
    }

    pub async fn post_purchase(&self, user_id: Uuid, payload: &PostPurchase) -> Result<(), AppError> {
        self.repo.post_purchase(user_id, payload).await
    }

    pub async fn post_sale(&self, user_id: Uuid, payload: &PostSale) -> Result<(), AppError> {
        self.repo.post_sale(user_id, payload).await
    }
}