use crate::{errors::AppError, models::vendor::{CreateVendor, Vendor}};
use uuid::Uuid;
use crate::features::vendors::repository::VendorRepository;

pub struct VendorService<R: VendorRepository> {
    repo: R,
}

impl<R: VendorRepository> VendorService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn create(&self, user_id: Uuid, payload: &CreateVendor) -> Result<Vendor, AppError> {
        self.repo.create(user_id, payload).await
    }
    pub async fn list(&self, user_id: Uuid) -> Result<Vec<Vendor>, AppError> {
        self.repo.list(user_id).await
    }
    pub async fn get(&self, id: i64, user_id: Uuid) -> Result<Vendor, AppError> {
        self.repo.get(id, user_id).await
    }
    pub async fn update(&self, id: i64, user_id: Uuid, payload: &CreateVendor) -> Result<Vendor, AppError> {
        self.repo.update(id, user_id, payload).await
    }
    pub async fn delete(&self, id: i64, user_id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id, user_id).await
    }
}