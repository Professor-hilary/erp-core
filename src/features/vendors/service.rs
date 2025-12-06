use crate::features::vendors::repository::VendorRepository;
use crate::{
    infrastructure::errors::AppError,
    models::vendor::{CreateVendor, Vendor},
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct VendorService<R: VendorRepository> {
    repo: R,
}

impl<R: VendorRepository> VendorService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError> {
        self.repo.create(tenant_pool, user_id, payload).await
    }
    pub async fn list(&self, tenant_pool: &PgPool, user_id: Uuid) -> Result<Vec<Vendor>, AppError> {
        self.repo.list(tenant_pool, user_id).await
    }
    pub async fn get(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vendor, AppError> {
        self.repo.get(tenant_pool, uuid, user_id).await
    }
    pub async fn update(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateVendor,
    ) -> Result<Vendor, AppError> {
        self.repo.update(tenant_pool, uuid, user_id, payload).await
    }
    pub async fn delete(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete(tenant_pool, uuid, user_id).await
    }
}
