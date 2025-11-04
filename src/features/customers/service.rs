use crate::{errors::AppError, models::customers::{CreateCustomer, Customer}};
use uuid::Uuid;
use crate::features::customers::repository::CustomerRepository;

pub struct CustomerService<R: CustomerRepository> {
    repo: R,
}

impl<R: CustomerRepository> CustomerService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn create(&self, user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError> {
        self.repo.create(user_id, payload).await
    }
    pub async fn list(&self, user_id: Uuid) -> Result<Vec<Customer>, AppError> {
        self.repo.list(user_id).await
    }
    pub async fn get(&self, id: i64, user_id: Uuid) -> Result<Customer, AppError> {
        self.repo.get(id, user_id).await
    }
    pub async fn update(&self, id: i64, user_id: Uuid, payload: &CreateCustomer) -> Result<Customer, AppError> {
        self.repo.update(id, user_id, payload).await
    }
    pub async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id, user_id).await
    }
}