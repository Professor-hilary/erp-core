// src/features/accounts/service.rs
use crate::{
    features::accounts::account_roles::{AccountRole, AccountRoleResolver},
    features::manufacturing::repository::ManufacturingRepo,
    interface::api::errors::AppError,
    models::manufacturing::*,
};
use uuid::Uuid;

pub struct ManufacturingService {
    repo: ManufacturingRepo,
}

impl ManufacturingService {
    pub fn new(repo: ManufacturingRepo) -> Self {
        Self { repo }
    }

    async fn resolver(&self) -> Result<AccountRoleResolver, AppError> {
        AccountRoleResolver::load(&self.repo.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to load account roles: {e}")))
    }

    fn role_ref(resolver: &AccountRoleResolver, role: AccountRole, override_code: Option<String>) -> Result<String, AppError> {
        if let Some(code) = override_code {
            if !code.is_empty() {
                return Ok(code);
            }
        }
        resolver
            .resolve_default(role)
            .map(|a| a.uuid.to_string())
            .map_err(|e| AppError::Unprocessable(format!(
                "Account role '{}' is not mapped: {e}",
                role.key()
            )))
    }

    pub async fn create_order(
        &self,
        dto: CreateProductionOrderDto,
    ) -> Result<ProductionOrder, AppError> {
        Ok(self.repo.create_production_order(dto).await?)
    }

    pub async fn new_work_center(&self, dto: WorkCenterDto) -> Result<WorkCenter, AppError> {
        Ok(self.repo.create_work_center(dto).await?)
    }

    pub async fn new_routing(&self, dto: RoutingsDto) -> Result<Routings, AppError> {
        Ok(self.repo.create_routings(dto).await?)
    }

    pub async fn new_routing_operation(
        &self,
        dto: RoutingOperationsDto,
    ) -> Result<RoutingOperations, AppError> {
        Ok(self.repo.create_routing_operation(dto).await?)
    }

    pub async fn create_overhead_rate(
        &self,
        dto: CreateOverheadRateDto,
    ) -> Result<OverheadRates, AppError> {
        Ok(self.repo.create_overhead_rate(dto).await?)
    }

    pub async fn bulk_issue_material(
        &self,
        dto: IssueMaterialDto,
        user_uuid: Uuid,
    ) -> Result<Vec<BomMaterialIssue>, AppError> {
        let resolver = self.resolver().await?;
        let dto = IssueMaterialDto {
            warehouse_serial_id: dto.warehouse_serial_id,
            production_order_uuid: dto.production_order_uuid,
            raw_mat_account_code: Some(Self::role_ref(
                &resolver, AccountRole::RawMaterialsInventory, dto.raw_mat_account_code,
            )?),
            wip_account_code: Some(Self::role_ref(
                &resolver, AccountRole::WorkInProgress, dto.wip_account_code,
            )?),
        };
        Ok(self.repo.issue_material_from_bom(dto, user_uuid).await?)
    }

    pub async fn issue_material(
        &self,
        resolver: &AccountRoleResolver,
        dto: SingleMaterialIssueDto,
        user_uuid: Uuid,
    ) -> Result<Vec<FullMaterialIssue>, AppError> {
        let raw_materials = resolver.resolve_default(AccountRole::RawMaterialsInventory)?;

        let work_in_progress = resolver.resolve_default(AccountRole::WorkInProgress)?;

        Ok(self
            .repo
            .issue_material_to_prod(dto, user_uuid, raw_materials.uuid, work_in_progress.uuid)
            .await?)
    }

    /// Overhead applied with overhead rate
    pub async fn apply_rated_overhead(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let resolver = self.resolver().await?;
        let dto = ApplyOverheadDto {
            production_order: dto.production_order,
            base_hours: dto.base_hours,
            allocation_base: dto.allocation_base,
            wip_account: Some(Self::role_ref(
                &resolver, AccountRole::WorkInProgress, dto.wip_account,
            )?),
            overhead_control_account: Some(Self::role_ref(
                &resolver, AccountRole::ManufacturingOverheadControl, dto.overhead_control_account,
            )?),
        };
        Ok(self.repo.apply_overhead_to_order(dto, user_uuid).await?)
    }

    /// Expensed overhead and cash or allocated
    pub async fn recognize_overhead_paid(
        &self,
        dto: RecognizeOverhead,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let resolver = self.resolver().await?;
        let cash = Self::role_ref(
            &resolver,
            AccountRole::CashAtBank,
            dto.payable_or_cash.clone(),
        ).or_else(|_| Self::role_ref(&resolver, AccountRole::AccountsPayable, dto.payable_or_cash))?;
        let dto = RecognizeOverhead {
            production_order_uuid: dto.production_order_uuid,
            base_amount: dto.base_amount,
            allocation_base: dto.allocation_base,
            reference: dto.reference,
            payable_or_cash: Some(cash),
            overhead_control_account: Some(Self::role_ref(
                &resolver, AccountRole::ManufacturingOverheadControl, dto.overhead_control_account,
            )?),
        };
        Ok(self.repo.recognize_actual_overhead(dto, user_uuid).await?)
    }

    // pub async fn apply_indirect_labor_costs(
    //     &self,
    //     dto: ApplyInDirectLaborDto,
    //     user_uuid: Uuid,
    // ) -> Result<CostApplication, AppError> {
    //     Ok(self.repo.apply_indirect_labor(dto, user_uuid).await?)
    // }

    pub async fn apply_direct_labor_costs(
        &self,
        dto: ApplyDirectLaborDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let resolver = self.resolver().await?;
        let dto = ApplyDirectLaborDto {
            production_order_uuid: dto.production_order_uuid,
            reference: dto.reference,
            labor_account_code: Some(Self::role_ref(
                &resolver, AccountRole::WagesPayable, dto.labor_account_code,
            )?),
            wip_account_code: Some(Self::role_ref(
                &resolver, AccountRole::WorkInProgress, dto.wip_account_code,
            )?),
        };
        Ok(self.repo.apply_direct_labor(dto, user_uuid).await?)
    }

    pub async fn start_order(
        &self,
        dto: StartProductionOrderDto,
    ) -> Result<ProductionOrder, AppError> {
        Ok(self
            .repo
            .start_production_order(dto.production_order_uuid)
            .await?)
    }

    pub async fn complete_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        let resolver = self.resolver().await?;
        let dto = CompleteProductionOrderDto {
            production_order_uuid: dto.production_order_uuid,
            completed_quantity: dto.completed_quantity,
            wip_account_code: Some(Self::role_ref(
                &resolver, AccountRole::WorkInProgress, dto.wip_account_code,
            )?),
            fg_account_code: Some(Self::role_ref(
                &resolver, AccountRole::FinishedGoodsInventory, dto.fg_account_code,
            )?),
            mfg_mat_var_code: Some(Self::role_ref(
                &resolver, AccountRole::MaterialUsageVariance, dto.mfg_mat_var_code,
            )?),
            mfg_lab_var_code: Some(Self::role_ref(
                &resolver, AccountRole::LabourEfficiencyVariance, dto.mfg_lab_var_code,
            )?),
            mfg_moh_var_code: Some(Self::role_ref(
                &resolver, AccountRole::OverheadVariance, dto.mfg_moh_var_code,
            )?),
            control_code: Some(Self::role_ref(
                &resolver, AccountRole::ManufacturingOverheadControl, dto.control_code,
            )?),
        };
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
        let std_cost = self
            .repo
            .calculate_standard_cost_for_item(&mut tx, header_dto.product_item_uuid)
            .await?;

        sqlx::query("UPDATE inventory.items SET standard_cost = $1 WHERE uuid = $2")
            .bind(std_cost.proration)
            .bind(header_dto.product_item_uuid)
            .execute(&mut *tx)
            .await?;

        // keep item.bom_uuid in sync when this is the default BOM
        if header_dto.is_default.unwrap_or(false) {
            sqlx::query("UPDATE inventory.items SET bom_uuid = $1 WHERE uuid = $2")
                .bind(header.uuid)
                .bind(header_dto.product_item_uuid)
                .execute(&mut *tx)
                .await?;
        }

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
