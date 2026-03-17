use crate::{
    features::sales::repository::CustomerRepository,
    interface::api::errors::AppError,
    models::customers::{
        ApplyPayment, CreateCustomer, CreateTurnover, Customer, Payment, PostTurnover, Turnover,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct CustomerService<R: CustomerRepository> {
    repo: R,
}

impl<R: CustomerRepository> CustomerService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError> {
        self.repo.create(tenant_pool, user_id, payload).await
    }

    pub async fn list(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Customer>, AppError> {
        self.repo.list(tenant_pool, user_id).await
    }

    pub async fn get(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Customer, AppError> {
        self.repo.get(tenant_pool, uuid, user_id).await
    }

    pub async fn update(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError> {
        self.repo.update(tenant_pool, uuid, user_id, payload).await
    }

    pub async fn delete(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        self.repo.delete(tenant_pool, uuid, user_id).await
    }

    // pub async fn create_sale(
    //     &self,
    //     tenant_pool: &PgPool,
    //     payload: &CreateTurnover,
    // ) -> Result<Turnover, AppError> {
    //     self.repo.create_sale_order(tenant_pool, payload).await
    // }

    pub async fn post_sale(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError> {
        self.repo.post_sale(tenant_pool, user_id, payload).await
    }

    pub async fn create_invoice(
        &self,
        tenant_pool: &PgPool,
        payload: &CreateTurnover,
    ) -> Result<Turnover, AppError> {
        self.repo.create_invoice(tenant_pool, payload).await
    }

    pub async fn post_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &PostTurnover,
    ) -> Result<Turnover, AppError> {
        self.repo.post_invoice(pool, user_id, payload).await
    }

    pub async fn list_customer_invoice(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
    ) -> Result<Vec<Turnover>, AppError> {
        self.repo.list_customer_invoices(pool, vendor_uuid).await
    }

    pub async fn apply_payment(
        &self,
        pool: &PgPool,
        cmd: ApplyPayment,
    ) -> Result<Payment, AppError> {
        self.repo.apply_payment(pool, cmd).await
    }
}
