// src/routes/vendors/service.rs
use crate::{
    features::procurement::repository::VendorRepository,
    interface::api::errors::AppError,
    models::{
        vendor::{
            ApplyPayment, CreatePurchase, CreateVendor, Payment, PostPurchase, Purchase, Vendor,
        },
    },
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

    pub async fn create_bill(
        &self,
        tenant_pool: &PgPool,
        payload: &CreatePurchase,
    ) -> Result<Purchase, AppError> {
        self.repo.create_bill(tenant_pool, payload).await
    }

    pub async fn post_bill(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<i64, AppError> {
        self.repo.post_bill(pool, user_id, payload).await
    }

    pub async fn post_cash_purchase(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostPurchase,
    ) -> Result<i64, AppError> {
        self.repo.post_purchase(pool, user_id, payload).await
    }

    pub async fn list_vendor_bills(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Purchase>, AppError> {
        self.repo.list_vendor_bills(pool, vendor_uuid).await
    }

    pub async fn apply_payment(
        &self,
        pool: &PgPool,
        cmd: ApplyPayment,
    ) -> Result<Payment, AppError> {
        self.repo.apply_payment(pool, cmd).await
    }
}
