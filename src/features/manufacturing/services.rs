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


    /// Stage A: load tenant costing policy (default standard). No posting behaviour change yet.
    pub async fn costing_policy(&self) -> Result<CostingPolicy, AppError> {
        self.repo.load_costing_policy().await
    }

    pub async fn update_costing_policy(
        &self,
        dto: UpdateCostingPolicyDto,
        updated_by: Uuid,
    ) -> Result<CostingPolicy, AppError> {
        self.repo.update_costing_policy(dto, updated_by).await
    }

    async fn resolver(&self) -> Result<AccountRoleResolver, AppError> {
        AccountRoleResolver::load(&self.repo.pool)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to load account roles: {e}")))
    }

    fn role_ref(
        resolver: &AccountRoleResolver,
        role: AccountRole,
        override_code: Option<String>,
    ) -> Result<String, AppError> {
        if let Some(code) = override_code {
            if !code.is_empty() {
                return Ok(code);
            }
        }
        resolver
            .resolve_default(role)
            .map(|a| a.uuid.to_string())
            .map_err(|e| {
                AppError::Unprocessable(format!("Account role '{}' is not mapped: {e}", role.key()))
            })
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
                &resolver,
                AccountRole::RawMaterialsInventory,
                dto.raw_mat_account_code,
            )?),
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
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
        let policy = self.costing_policy().await?;
        if !policy.allows_predetermined_overhead() {
            return Err(AppError::Unprocessable(format!(
                "Predetermined overhead application is not used under actual costing (current method: {}).                  Recognize actual overhead instead.",
                policy.method().as_str()
            )));
        }
        let resolver = self.resolver().await?;
        let dto = ApplyOverheadDto {
            production_order: dto.production_order,
            base_hours: dto.base_hours,
            allocation_base: dto.allocation_base,
            wip_account: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account,
            )?),
            overhead_control_account: Some(Self::role_ref(
                &resolver,
                AccountRole::ManufacturingOverheadControl,
                dto.overhead_control_account,
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
        )
        .or_else(|_| {
            Self::role_ref(&resolver, AccountRole::AccountsPayable, dto.payable_or_cash)
        })?;
        let dto = RecognizeOverhead {
            production_order_uuid: dto.production_order_uuid,
            base_amount: dto.base_amount,
            allocation_base: dto.allocation_base,
            reference: dto.reference,
            payable_or_cash: Some(cash),
            overhead_control_account: Some(Self::role_ref(
                &resolver,
                AccountRole::ManufacturingOverheadControl,
                dto.overhead_control_account,
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

    pub async fn create_labour_category(
        &self,
        dto: CreateLabourCategoryDto,
    ) -> Result<LabourCategory, AppError> {
        Ok(self.repo.create_labour_category(dto).await?)
    }

    pub async fn list_labour_categories(&self) -> Result<Vec<LabourCategory>, AppError> {
        Ok(self.repo.list_labour_categories().await?)
    }

    pub async fn create_labour_rate(
        &self,
        dto: CreateLabourRateDto,
    ) -> Result<LabourRate, AppError> {
        Ok(self.repo.create_labour_rate(dto).await?)
    }

    pub async fn create_labour_time_entry(
        &self,
        dto: CreateLabourTimeEntryDto,
        user_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        Ok(self.repo.create_labour_time_entry(dto, user_uuid).await?)
    }

    pub async fn submit_labour_time_entry(
        &self,
        entry_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        Ok(self.repo.submit_labour_time_entry(entry_uuid).await?)
    }

    pub async fn approve_labour_time_entry(
        &self,
        dto: ApproveLabourTimeEntryDto,
        approver: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        Ok(self.repo.approve_labour_time_entry(dto, approver).await?)
    }

    pub async fn reject_labour_time_entry(
        &self,
        dto: RejectLabourTimeEntryDto,
    ) -> Result<LabourTimeEntry, AppError> {
        Ok(self.repo.reject_labour_time_entry(dto).await?)
    }

    pub async fn post_labour_time_entry(
        &self,
        dto: PostLabourTimeEntryDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.allows_timesheet_actual() {
            return Err(AppError::Unprocessable(
                "Timesheet actual labour is disabled on the costing policy (use_timesheet_actual=false)."
                    .into(),
            ));
        }

        let resolver = self.resolver().await?;
        let dto = PostLabourTimeEntryDto {
            entry_uuid: dto.entry_uuid,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            wages_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.wages_account_code,
            )?),
            overhead_control_account: dto.overhead_control_account.filter(|s| !s.is_empty()),
        };
        Ok(self.repo.post_labour_time_entry(dto, user_uuid).await?)
    }

    pub async fn draft_time_from_operation(
        &self,
        dto: DraftTimeFromOperationDto,
        user_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        Ok(self.repo.draft_time_from_operation(dto, user_uuid).await?)
    }

    pub async fn list_labour_time_entries_for_order(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        Ok(self
            .repo
            .list_labour_time_entries_for_order(order_uuid)
            .await?)
    }

    pub async fn create_team_time_entries(
        &self,
        dto: CreateTeamTimeEntriesDto,
        user_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        Ok(self.repo.create_team_time_entries(dto, user_uuid).await?)
    }

    pub async fn bulk_submit_time_entries(&self, entry_uuids: Vec<Uuid>) -> Result<i32, AppError> {
        Ok(self.repo.bulk_submit_time_entries(&entry_uuids).await?)
    }

    pub async fn bulk_approve_time_entries(
        &self,
        dto: BulkTimeEntryIdsDto,
        approver: Uuid,
    ) -> Result<i32, AppError> {
        Ok(self
            .repo
            .bulk_approve_time_entries(&dto.entry_uuids, approver, dto.hourly_rate)
            .await?)
    }

    pub async fn bulk_post_time_entries(
        &self,
        dto: BulkTimeEntryIdsDto,
        user_uuid: Uuid,
    ) -> Result<i32, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.allows_timesheet_actual() {
            return Err(AppError::Unprocessable(
                "Timesheet actual labour is disabled on the costing policy (use_timesheet_actual=false)."
                    .into(),
            ));
        }

        let resolver = self.resolver().await?;
        let dto = BulkTimeEntryIdsDto {
            entry_uuids: dto.entry_uuids,
            hourly_rate: dto.hourly_rate,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            wages_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.wages_account_code,
            )?),
            overhead_control_account: dto.overhead_control_account,
        };
        Ok(self.repo.bulk_post_time_entries(dto, user_uuid).await?)
    }

    pub async fn allocate_payrun_labour(
        &self,
        dto: AllocatePayrunLabourDto,
        user_uuid: Uuid,
    ) -> Result<Vec<PayrunLabourAllocationRow>, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.allows_payroll_allocation() {
            return Err(AppError::Unprocessable(
                "Payroll labour allocation is disabled on the costing policy (use_payroll_allocation=false)."
                    .into(),
            ));
        }

        let resolver = self.resolver().await?;
        let dto = AllocatePayrunLabourDto {
            payrun_uuid: dto.payrun_uuid,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            labour_clearing_account: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.labour_clearing_account,
            )?),
            cost_basis: dto.cost_basis,
        };
        Ok(self.repo.allocate_payrun_labour(dto, user_uuid).await?)
    }

    pub async fn draft_time_from_order_ops(
        &self,
        order_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        Ok(self
            .repo
            .draft_time_from_order_ops(order_uuid, user_uuid)
            .await?)
    }

    pub async fn upsert_attendance(
        &self,
        dto: UpsertAttendanceDto,
        user_uuid: Uuid,
    ) -> Result<AttendanceEntry, AppError> {
        Ok(self.repo.upsert_attendance(dto, user_uuid).await?)
    }

    pub async fn labour_hours_reconciliation(
        &self,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<LabourHoursReconciliation>, AppError> {
        Ok(self.repo.labour_hours_reconciliation(employee_uuid).await?)
    }

    pub async fn labour_hours_reconciliation_period(
        &self,
    ) -> Result<Vec<LabourHoursReconciliationPeriod>, AppError> {
        Ok(self.repo.labour_hours_reconciliation_period().await?)
    }

    pub async fn allocate_payslip_labour(
        &self,
        dto: AllocatePayslipLabourDto,
        user_uuid: Uuid,
    ) -> Result<Vec<PayslipLabourAllocationRow>, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.allows_payroll_allocation() {
            return Err(AppError::Unprocessable(
                "Payroll labour allocation is disabled on the costing policy (use_payroll_allocation=false)."
                    .into(),
            ));
        }

        let resolver = self.resolver().await?;
        let dto = AllocatePayslipLabourDto {
            payslip_uuid: dto.payslip_uuid,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            labour_clearing_account: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.labour_clearing_account,
            )?),
            cost_basis: dto.cost_basis,
        };
        Ok(self.repo.allocate_payslip_labour(dto, user_uuid).await?)
    }

    pub async fn labour_productivity_by_order(
        &self,
        order_uuid: Option<Uuid>,
    ) -> Result<Vec<LabourProductivityByOrder>, AppError> {
        Ok(self.repo.labour_productivity_by_order(order_uuid).await?)
    }

    pub async fn labour_by_employee(&self) -> Result<Vec<LabourByEmployee>, AppError> {
        Ok(self.repo.labour_by_employee().await?)
    }

    pub async fn labour_by_work_center(&self) -> Result<Vec<LabourByWorkCenter>, AppError> {
        Ok(self.repo.labour_by_work_center().await?)
    }

    pub async fn operation_efficiency(
        &self,
        order_uuid: Option<Uuid>,
    ) -> Result<Vec<OperationEfficiency>, AppError> {
        Ok(self.repo.operation_efficiency(order_uuid).await?)
    }

    pub async fn compute_labour_variances(
        &self,
        order_uuid: Uuid,
        as_of: Option<chrono::NaiveDate>,
    ) -> Result<LabourVariance, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.posts_standard_variances() {
            return Err(AppError::Unprocessable(format!(
                "Standard labour variance analysis applies under standard costing only (current method: {}).",
                policy.method().as_str()
            )));
        }

        Ok(self
            .repo
            .compute_labour_variances(order_uuid, as_of)
            .await?)
    }

    pub async fn production_order_labour_summary(
        &self,
        order_uuid: Uuid,
    ) -> Result<ProductionOrderLabourSummary, AppError> {
        Ok(self
            .repo
            .production_order_labour_summary(order_uuid)
            .await?)
    }

    pub async fn create_labour_allocation(
        &self,
        dto: CreateLabourAllocationDto,
        user_uuid: Uuid,
    ) -> Result<LabourAllocation, AppError> {
        Ok(self.repo.create_labour_allocation(dto, user_uuid).await?)
    }

    pub async fn post_labour_allocation(
        &self,
        dto: PostLabourAllocationDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let resolver = self.resolver().await?;
        let dto = PostLabourAllocationDto {
            allocation_uuid: dto.allocation_uuid,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            wages_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.wages_account_code,
            )?),
            overhead_control_account: match dto.overhead_control_account {
                Some(c) if !c.is_empty() => Some(c),
                _ => None,
            },
        };
        Ok(self.repo.post_labour_allocation(dto, user_uuid).await?)
    }

    pub async fn ensure_operation_executions(
        &self,
        dto: EnsureOperationsDto,
    ) -> Result<i32, AppError> {
        Ok(self
            .repo
            .ensure_operation_executions(dto.production_order_uuid)
            .await?)
    }

    pub async fn list_operation_executions(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<OperationExecution>, AppError> {
        Ok(self.repo.list_operation_executions(order_uuid).await?)
    }

    pub async fn start_operation(
        &self,
        dto: StartOperationDto,
    ) -> Result<OperationExecution, AppError> {
        Ok(self.repo.start_operation(dto).await?)
    }

    pub async fn complete_operation(
        &self,
        dto: CompleteOperationDto,
    ) -> Result<OperationExecution, AppError> {
        Ok(self.repo.complete_operation(dto).await?)
    }

    pub async fn record_scrap_rework(
        &self,
        dto: RecordScrapReworkDto,
        user_uuid: Uuid,
    ) -> Result<ScrapReworkEvent, AppError> {
        Ok(self.repo.record_scrap_rework(dto, user_uuid).await?)
    }

    pub async fn record_quality_inspection(
        &self,
        dto: RecordQualityInspectionDto,
        user_uuid: Uuid,
    ) -> Result<QualityInspection, AppError> {
        Ok(self.repo.record_quality_inspection(dto, user_uuid).await?)
    }

    pub async fn apply_direct_labor_costs(
        &self,
        dto: ApplyDirectLaborDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let policy = self.costing_policy().await?;
        if !policy.allows_routing_applied_labor() {
            return Err(AppError::Unprocessable(format!(
                "Routing applied labour is only used under standard costing (current method: {}).                  Post actual labour via timesheets or payroll allocation instead.",
                policy.method().as_str()
            )));
        }
        let resolver = self.resolver().await?;
        let dto = ApplyDirectLaborDto {
            production_order_uuid: dto.production_order_uuid,
            reference: dto.reference,
            labor_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WagesPayable,
                dto.labor_account_code,
            )?),
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
        };
        Ok(self.repo.apply_direct_labor(dto, user_uuid).await?)
    }

    pub async fn start_order(
        &self,
        dto: StartProductionOrderDto,
    ) -> Result<ProductionOrder, AppError> {
        let order = self
            .repo
            .start_production_order(dto.production_order_uuid)
            .await?;
        // Seed operation execution rows from pinned/default routing
        let _ = self
            .repo
            .ensure_operation_executions(dto.production_order_uuid)
            .await?;
        Ok(order)
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
            fg_warehouse_serial: dto.fg_warehouse_serial,
            wip_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::WorkInProgress,
                dto.wip_account_code,
            )?),
            fg_account_code: Some(Self::role_ref(
                &resolver,
                AccountRole::FinishedGoodsInventory,
                dto.fg_account_code,
            )?),
            mfg_mat_var_code: Some(Self::role_ref(
                &resolver,
                AccountRole::MaterialUsageVariance,
                dto.mfg_mat_var_code,
            )?),
            mfg_lab_var_code: Some(Self::role_ref(
                &resolver,
                AccountRole::LabourEfficiencyVariance,
                dto.mfg_lab_var_code,
            )?),
            mfg_moh_var_code: Some(Self::role_ref(
                &resolver,
                AccountRole::OverheadVariance,
                dto.mfg_moh_var_code,
            )?),
            control_code: Some(Self::role_ref(
                &resolver,
                AccountRole::ManufacturingOverheadControl,
                dto.control_code,
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
