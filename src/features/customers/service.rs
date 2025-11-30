use crate::features::customers::repository::CustomerRepository;
use crate::{
    infrastructure::errors::AppError,
    models::customers::{CreateCustomer, Customer},
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
        self.repo.create(tenant_pool,user_id, payload).await
    }
    pub async fn list(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Customer>, AppError> {
        self.repo.list(tenant_pool,user_id).await
    }
    pub async fn get(
        &self,
        tenant_pool: &PgPool,
        id: i64,
        user_id: Uuid,
    ) -> Result<Customer, AppError> {
        self.repo.get(tenant_pool,id, user_id).await
    }
    pub async fn update(
        &self,
        tenant_pool: &PgPool,
        id: i64,
        user_id: Uuid,
        payload: &CreateCustomer,
    ) -> Result<Customer, AppError> {
        self.repo.update(tenant_pool,id, user_id, payload).await
    }
    pub async fn delete(
        &self,
        tenant_pool: &PgPool,
        id: i64,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete(tenant_pool,id, user_id).await
    }
}
