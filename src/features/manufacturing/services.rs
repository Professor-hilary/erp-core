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
    pub async fn create_bom(
        &self,
        header_dto: CreateBomHeaderDto,
        lines: Vec<CreateBomLineDto>,
        user_uuid: Uuid,
    ) -> Result<BomWithLines, anyhow::Error> {
        let mut tx = self.repo.pool.begin().await?;

        let header = sqlx::query_as::<_, BomHeader>(r#""#)
            .fetch_one(&mut *tx)
            .await?;

        let mut created_lines = vec![];

        for (idx, line_dto) in lines.into_iter().enumerate() {
            let line = sqlx::query_as::<_, BomLine>(r#""#)
                .fetch_one(&mut *tx)
                .await?;
            created_lines.push(line);
        }

        tx.commit().await?;

        Ok(BomWithLines {
            header,
            lines: created_lines,
        })
    }

    pub async fn get_bom(&self, bom_uuid: Uuid) -> Result<Option<BomWithLines>, anyhow::Error> {
        Ok(self.repo.get_full_bom(bom_uuid).await?)
    }
}
