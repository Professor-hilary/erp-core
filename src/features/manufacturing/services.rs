// src/features/accounts/service.rs
use crate::{
    features::manufacturing::repository::ManufacturingRepo, interface::api::errors::AppError,
    models::manufacturing::*,
};
use bigdecimal::BigDecimal;
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
    ) -> Result<ProductionOrder, AppError> {
        Ok(self.repo.create_production_order(dto).await?)
    }

    pub async fn create_overhead_rate(
        &self,
        dto: CreateOverheadRateDto,
    ) -> Result<OverheadRates, AppError> {
        Ok(self.repo.create_overhead_rate(dto).await?)
    }

    pub async fn issue_material(
        &self,
        dto: IssueMaterialDto,
        user_uuid: Uuid,
    ) -> Result<MaterialIssue, AppError> {
        Ok(self.repo.issue_material(dto, user_uuid).await?)
    }

    pub async fn apply_overhead(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        Ok(self.repo.apply_overhead_to_order(dto, user_uuid).await?)
    }

    pub async fn recognize_expenditures(
        &self,
        dto: RecognizeOverhead,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        Ok(self.repo.recognize_actual_overhead(dto, user_uuid).await?)
    }

    pub async fn apply_labor_costs(
        &self,
        dto: ApplyLaborCostDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        Ok(self.repo.apply_labor_costs(dto, user_uuid).await?)
    }

    pub async fn complete_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        Ok(self.repo.complete_production_order(dto, user_uuid).await?)
    }

    pub async fn prorate_variance(
        &self,
        dto: ProrateVarianceDto,
        user_uuid: Uuid,
    ) -> Result<VarianceProrationResult, AppError> {
        Ok(self.repo.prorate_variance(dto, user_uuid).await?)
    }

    pub async fn create_bom(
        &self,
        header_dto: CreateBomHeaderDto,
        lines: Vec<CreateBomLineDto>,
        user_uuid: Uuid,
    ) -> Result<BomWithLines, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = self.repo.pool.begin().await?;

        let header: BomHeader = self
            .repo
            .create_header(&mut tx, &header_dto, user_uuid)
            .await?;

        let mut created_lines: Vec<BomLine> = Vec::with_capacity(lines.len());

        for line_dto in &lines {
            let line: BomLine = self.repo.add_line(&mut tx, header.uuid, line_dto).await?;
            created_lines.push(line);
        }

        // Calculate standard cost
        let std_cost: BigDecimal = self
            .repo
            .calculate_standard_cost_for_item(&mut tx, header_dto.product_item_uuid)
            .await?;

        sqlx::query("UPDATE inventory.items SET standard_cost = $1 WHERE uuid = $2")
            .bind(std_cost)
            .bind(header_dto.product_item_uuid)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(BomWithLines {
            header,
            lines: created_lines,
        })
    }

    pub async fn get_bom(&self, bom_uuid: Uuid) -> Result<Option<BomWithLines>, AppError> {
        Ok(self.repo.get_full_bom(bom_uuid).await?)
    }

    pub async fn default_bom(&self, uuid: Uuid) -> Result<Option<BomWithLines>, AppError> {
        Ok(self.repo.get_default_bom_for_product(uuid).await?)
    }
}
