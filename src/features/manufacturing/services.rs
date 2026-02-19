// src/features/accounts/service.rs
use crate::{features::manufacturing::repository::ManufacturingRepo, models::manufacturing::*};
use uuid::Uuid;

pub struct ManufacturingService {
    repo: ManufacturingRepo,
}

impl ManufacturingService {
    pub fn new(repo: ManufacturingRepo) -> Self {
        Self { repo }
    }

    pub async fn create_order(
        &self,
        dto: CreateProductionOrderDto,
    ) -> Result<ProductionOrder, anyhow::Error> {
        Ok(self.repo.create_production_order(dto).await?)
    }

    pub async fn issue_material(
        &self,
        dto: IssueMaterialDto,
        user_uuid: Uuid,
    ) -> Result<MaterialIssue, anyhow::Error> {
        Ok(self.repo.issue_material(dto, user_uuid).await?)
    }

    pub async fn apply_overhead(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, anyhow::Error> {
        Ok(self.repo.apply_overhead(dto, user_uuid).await?)
    }

    pub async fn complete_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, anyhow::Error> {
        Ok(self.repo.complete_production_order(dto, user_uuid).await?)
    }

    pub async fn prorate_variance(
        &self,
        dto: ProrateVarianceDto,
        user_uuid: Uuid,
    ) -> Result<VarianceProrationResult, anyhow::Error> {
        Ok(self.repo.prorate_variance_v2(dto, user_uuid).await?)
    }
}
