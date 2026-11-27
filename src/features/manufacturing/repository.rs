// src/features/manufacturing/repositories.rs
use crate::{interface::api::errors::AppError, models::manufacturing::*};
use bigdecimal::{BigDecimal, One};
use chrono::prelude::NaiveDate;
use serde::Deserialize;
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;
pub struct ManufacturingRepo {
    pub(crate) pool: PgPool,
}

#[derive(Debug, Deserialize, FromRow, sqlx::Encode)]
pub struct Cost {
    pub proration: BigDecimal,
}

impl ManufacturingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================ Production Order ============================
    pub async fn create_production_order(
        &self,
        dto: CreateProductionOrderDto,
    ) -> Result<ProductionOrder, AppError> {
        let mut tx = self.pool.begin().await?;

        // Pin default active BOM + routing and freeze standard unit cost at release
        let order = sqlx::query_as::<_, ProductionOrder>(
            r#"
            WITH pinned AS (
                SELECT
                    (SELECT bh.uuid
                     FROM manufacturing.bom_headers bh
                     WHERE bh.product_item_uuid = $2
                       AND bh.is_active
                     ORDER BY bh.is_default DESC, bh.created_at DESC
                     LIMIT 1) AS bom_uuid,
                    (SELECT r.uuid
                     FROM manufacturing.routings r
                     WHERE r.product_item_uuid = $2
                       AND r.status = 'active'
                     ORDER BY r.is_default DESC, r.created_at DESC
                     LIMIT 1) AS routing_uuid
            )
            INSERT INTO manufacturing.production_orders(
                order_number, product_item_uuid, quantity_ordered, start_date,
                expected_completion_date, status,
                bom_header_uuid, routing_uuid, standard_unit_cost
            )
            SELECT
                $1, $2, $3, $4, $5, 'Planned',
                pinned.bom_uuid,
                pinned.routing_uuid,
                CASE WHEN pinned.bom_uuid IS NOT NULL
                     THEN manufacturing.calculate_standard_cost($2, COALESCE($4, CURRENT_DATE))
                     ELSE NULL
                END
            FROM pinned
            RETURNING *
            "#,
        )
        .bind(&dto.order_number)
        .bind(dto.product_item_uuid)
        .bind(&dto.quantity_ordered)
        .bind(dto.start_date)
        .bind(dto.expected_completion_date)
        .fetch_one(&mut *tx)
        .await?;

        if order.bom_header_uuid.is_none() {
            tx.rollback().await.ok();
            return Err(AppError::Unprocessable(format!(
                "No active BOM for product {} — cannot release production order",
                dto.product_item_uuid
            )));
        }

        // Explode the *pinned* BOM (not a later default)
        sqlx::query(
            r#"
            INSERT INTO manufacturing.production_order_materials(
                production_order_uuid, bom_line_uuid, component_item_uuid, required_qty
            )
            SELECT
                p.uuid,
                NULL,
                e.component_item_uuid,
                SUM(e.quantity_required)
            FROM manufacturing.production_orders p
            CROSS JOIN LATERAL manufacturing.explode_bom(p.bom_header_uuid, p.quantity_ordered) e
            WHERE p.uuid = $1
              AND e.is_leaf = true
              AND COALESCE(e.is_phantom, false) = false
            GROUP BY p.uuid, e.component_item_uuid
            "#,
        )
        .bind(order.uuid)
        .execute(&mut *tx)
        .await?;

        let n: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM manufacturing.production_order_materials
            WHERE production_order_uuid = $1
            "#,
        )
        .bind(order.uuid)
        .fetch_one(&mut *tx)
        .await?;
        if n == 0 {
            tx.rollback().await.ok();
            return Err(AppError::Unprocessable(format!(
                "Production order {} has no leaf materials after BOM explode",
                order.uuid
            )));
        }

        tx.commit().await?;
        Ok(order)
    }

    pub async fn add_line(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        header_uuid: Uuid,
        dto: &CreateBomLineDto,
    ) -> Result<BomLine, AppError> {
        // Only run cycle detection when we have a concrete BOM to explode
        let has_bom_to_check: bool = dto.child_bom_uuid.is_some()
            || sqlx::query_scalar::<_, bool>(
                r#"
            SELECT EXISTS (
                SELECT 1 FROM manufacturing.bom_headers
                WHERE product_item_uuid = $1 AND is_active
            )
            "#,
            )
            .bind(dto.component_item_uuid)
            .fetch_one(&mut **tx)
            .await
            .unwrap_or(false);

        if has_bom_to_check {
            // Reject cycles before insert
            let cycle: bool = sqlx::query_scalar(
                r#"
            SELECT EXISTS (
                SELECT 1
                FROM manufacturing.explode_bom(
                    COALESCE($1, (
                        SELECT uuid FROM manufacturing.bom_headers
                        WHERE product_item_uuid = $2 AND is_active
                        ORDER BY is_default DESC, created_at DESC
                        LIMIT 1
                    )),
                    1
                ) e
                WHERE e.component_item_uuid = (
                    SELECT product_item_uuid FROM manufacturing.bom_headers WHERE uuid = $3
                )
            )
            "#,
            )
            .bind(dto.child_bom_uuid)
            .bind(dto.component_item_uuid)
            .bind(header_uuid)
            .fetch_one(&mut **tx)
            .await
            .unwrap_or(false);

            if cycle {
                return Err(AppError::BadRequest(
                    "Circular BOM: component eventually contains this product".into(),
                ));
            }
        }

        let bom_line = sqlx::query_as::<_, BomLine>(
            r#"
            INSERT INTO manufacturing.bom_lines (
                bom_header_uuid, line_number, component_item_uuid,
                quantity_per, uom, scrap_factor, notes,
                child_bom_uuid, is_phantom
            ) VALUES (
                $1, $2, $3, $4,
                COALESCE($5, 'pcs'), COALESCE($6, 1.0000), $7,
                $8, COALESCE($9, false)
            )
            RETURNING *
            "#,
        )
        .bind(header_uuid)
        .bind(dto.line_number)
        .bind(dto.component_item_uuid)
        .bind(dto.quantity_per.clone())
        .bind(dto.uom.as_deref())
        .bind(dto.scrap_factor.clone().unwrap_or(BigDecimal::one()))
        .bind(dto.notes.as_deref())
        .bind(dto.child_bom_uuid)
        .bind(dto.is_phantom.unwrap_or(false))
        .fetch_one(&mut **tx)
        .await?;

        Ok(bom_line)
    }

    pub async fn create_work_center(&self, dto: WorkCenterDto) -> Result<WorkCenter, AppError> {
        let work_center = sqlx::query_as::<_, WorkCenter>(
            r#"
            INSERT INTO manufacturing.work_centers(
                code, name, labor_rate, allocation_base, department_code
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&dto.code)
        .bind(&dto.name)
        .bind(dto.labor_rate)
        .bind(dto.allocation_base)
        .bind(dto.department_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(work_center)
    }

    pub async fn create_routings(&self, dto: RoutingsDto) -> Result<Routings, AppError> {
        let routing = sqlx::query_as::<_, Routings>(
            r#"
            INSERT INTO manufacturing.routings(
                product_item_uuid, routing_code, notes, version,
                base_quantity, effective_date, status, is_default
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(&dto.product_item_uuid)
        .bind(dto.routing_code)
        .bind(dto.notes)
        .bind(dto.version)
        .bind(dto.base_quantity)
        .bind(dto.effective_date)
        .bind(dto.status)
        .bind(dto.is_default)
        .fetch_one(&self.pool)
        .await?;

        Ok(routing)
    }

    pub async fn create_routing_operation(
        &self,
        dto: RoutingOperationsDto,
    ) -> Result<RoutingOperations, AppError> {
        let routing = sqlx::query_as::<_, RoutingOperations>(
            r#"
            INSERT INTO manufacturing.routing_operations(
                routing_uuid, sequence, operation_name, work_center,
                description, setup_time_minutes, run_time_minutes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&dto.routing_uuid)
        .bind(dto.sequence)
        .bind(dto.operation_name)
        .bind(dto.work_center)
        .bind(dto.description)
        .bind(dto.setup_time_minutes)
        .bind(dto.run_time_minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(routing)
    }

    pub async fn create_overhead_rate(
        &self,
        dto: CreateOverheadRateDto,
    ) -> Result<OverheadRates, AppError> {
        let overhead_rates = sqlx::query_as::<_, OverheadRates>(
            r#"
            INSERT INTO manufacturing.overhead_rates(
                period_start, period_end, allocation_base, estimated_overhead,
                estimated_base, department_code, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&dto.period_start)
        .bind(dto.period_end)
        .bind(dto.allocation_base)
        .bind(dto.estimated_overhead)
        .bind(dto.estimated_base)
        .bind(&dto.department_code)
        .bind(dto.is_active)
        .fetch_one(&self.pool)
        .await?;

        // Update cost tracker to record material cost
        Ok(overhead_rates)
    }

    pub async fn get_production_order(&self, uuid: Uuid) -> Result<ProductionOrder, AppError> {
        let production_order: ProductionOrder = sqlx::query_as::<_, ProductionOrder>(
            "SELECT * FROM manufacturing.production_orders WHERE uuid = $1",
        )
        .bind(uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(production_order)
    }

    // ============================ Material Issue ============================
    pub async fn issue_material_from_bom(
        &self,
        dto: IssueMaterialDto,
        user_uuid: Uuid,
    ) -> Result<Vec<BomMaterialIssue>, AppError> {
        let materials: Vec<BomMaterialIssue> = sqlx::query_as::<_, BomMaterialIssue>(
            r#"SELECT * FROM manufacturing.issue_material_from_bom($1, $2, $3, $4, $5)"#,
        )
        .bind(&dto.production_order_uuid)
        .bind(dto.warehouse_serial_id)
        .bind(dto.raw_mat_account_code.as_deref().unwrap_or(""))
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(materials)
    }

    pub async fn issue_material_to_prod(
        &self,
        dto: SingleMaterialIssueDto,
        user_uuid: Uuid,
        raw_materials_account: Uuid,
        wip_account: Uuid,
    ) -> Result<Vec<FullMaterialIssue>, AppError> {
        let materials: Vec<FullMaterialIssue> = sqlx::query_as::<_, FullMaterialIssue>(
            r#"SELECT * FROM manufacturing.issue_material_to_order($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(dto.item_serial_id)
        .bind(dto.warehouse_serial)
        .bind(dto.quantity)
        .bind(&dto.production_order_uuid)
        .bind(raw_materials_account)
        .bind(wip_account)
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(materials)
    }

    // ============================ Recognize Rated Overhead (Non-Actual) ============================
    pub async fn apply_overhead_to_order(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let cost_application: CostApplication = sqlx::query_as::<_, CostApplication>(
            "
                SELECT * FROM manufacturing.apply_overhead_to_order(
                    $1, $2, $3, $4, $5, $6
                )
            ",
        )
        .bind(dto.production_order)
        .bind(dto.base_hours)
        .bind(dto.wip_account.as_deref().unwrap_or(""))
        .bind(dto.overhead_control_account.as_deref().unwrap_or(""))
        .bind(dto.allocation_base)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(cost_application)
    }

    /// Recognize Actual Overhead Expenditure
    /// Use real amounts - not rates for indirect overheads
    pub async fn recognize_actual_overhead(
        &self,
        dto: RecognizeOverhead,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let cost_application = sqlx::query_as::<_, CostApplication>(
            "
                SELECT * FROM manufacturing.record_actual_overhead(
                    $1, $2, $3, $4, $5, $6, $7
                )
            ",
        )
        .bind(dto.production_order_uuid)
        .bind(dto.base_amount)
        .bind(dto.payable_or_cash.as_deref().unwrap_or(""))
        .bind(dto.overhead_control_account.as_deref().unwrap_or(""))
        .bind(dto.allocation_base)
        .bind(dto.reference)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(cost_application)
    }

    // =============================== Labor Application ============================

    pub async fn create_labour_category(
        &self,
        dto: CreateLabourCategoryDto,
    ) -> Result<LabourCategory, AppError> {
        let row = sqlx::query_as::<_, LabourCategory>(
            r#"
            INSERT INTO manufacturing.labour_categories (code, name, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&dto.code)
        .bind(&dto.name)
        .bind(dto.description.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_labour_categories(&self) -> Result<Vec<LabourCategory>, AppError> {
        let rows = sqlx::query_as::<_, LabourCategory>(
            "SELECT * FROM manufacturing.labour_categories WHERE is_active ORDER BY code",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_labour_rate(
        &self,
        dto: CreateLabourRateDto,
    ) -> Result<LabourRate, AppError> {
        let row = sqlx::query_as::<_, LabourRate>(
            r#"
            INSERT INTO manufacturing.labour_rates (
                labour_category_uuid, employee_uuid, work_center_code,
                rate_kind, hourly_rate, effective_from, effective_to, notes
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            RETURNING *
            "#,
        )
        .bind(dto.labour_category_uuid)
        .bind(dto.employee_uuid)
        .bind(dto.work_center_code.as_deref())
        .bind(&dto.rate_kind)
        .bind(&dto.hourly_rate)
        .bind(dto.effective_from)
        .bind(dto.effective_to)
        .bind(dto.notes.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_labour_time_entry(
        &self,
        dto: CreateLabourTimeEntryDto,
        user_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        let row = sqlx::query_as::<_, LabourTimeEntry>(
            r#"
            SELECT * FROM manufacturing.create_labour_time_entry(
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15
            )
            "#,
        )
        .bind(dto.employee_uuid)
        .bind(dto.work_date)
        .bind(dto.hours)
        .bind(dto.activity_type.as_deref().unwrap_or("Direct"))
        .bind(dto.production_order_uuid)
        .bind(dto.operation_execution_uuid)
        .bind(dto.work_center_code.as_deref())
        .bind(dto.labour_category_uuid)
        .bind(dto.start_time)
        .bind(dto.end_time)
        .bind(dto.break_minutes)
        .bind(dto.quantity_completed)
        .bind(dto.quantity_scrapped)
        .bind(dto.memo.as_deref())
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn submit_labour_time_entry(
        &self,
        entry_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        let row = sqlx::query_as::<_, LabourTimeEntry>(
            "SELECT * FROM manufacturing.submit_labour_time_entry($1)",
        )
        .bind(entry_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn approve_labour_time_entry(
        &self,
        dto: ApproveLabourTimeEntryDto,
        approver: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        let row = sqlx::query_as::<_, LabourTimeEntry>(
            "SELECT * FROM manufacturing.approve_labour_time_entry($1, $2, $3)",
        )
        .bind(dto.entry_uuid)
        .bind(approver)
        .bind(dto.hourly_rate)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn reject_labour_time_entry(
        &self,
        dto: RejectLabourTimeEntryDto,
    ) -> Result<LabourTimeEntry, AppError> {
        let row = sqlx::query_as::<_, LabourTimeEntry>(
            "SELECT * FROM manufacturing.reject_labour_time_entry($1, $2)",
        )
        .bind(dto.entry_uuid)
        .bind(dto.reason.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn post_labour_time_entry(
        &self,
        dto: PostLabourTimeEntryDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let row = sqlx::query_as::<_, CostApplication>(
            "SELECT * FROM manufacturing.post_labour_time_entry($1,$2,$3,$4,$5)",
        )
        .bind(dto.entry_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.wages_account_code.as_deref().unwrap_or(""))
        .bind(dto.overhead_control_account.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn draft_time_from_operation(
        &self,
        dto: DraftTimeFromOperationDto,
        user_uuid: Uuid,
    ) -> Result<LabourTimeEntry, AppError> {
        let row = sqlx::query_as::<_, LabourTimeEntry>(
            "SELECT * FROM manufacturing.draft_time_entry_from_operation($1,$2,$3)",
        )
        .bind(dto.execution_uuid)
        .bind(dto.employee_uuid)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_labour_time_entries_for_order(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        let rows = sqlx::query_as::<_, LabourTimeEntry>(
            r#"
            SELECT * FROM manufacturing.labour_time_entries
            WHERE production_order_uuid = $1
            ORDER BY work_date, created_at
            "#,
        )
        .bind(order_uuid)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_team_time_entries(
        &self,
        dto: CreateTeamTimeEntriesDto,
        user_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        let rows = sqlx::query_as::<_, LabourTimeEntry>(
            r#"
            SELECT * FROM manufacturing.create_team_time_entries(
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10
            )
            "#,
        )
        .bind(&dto.employee_uuids)
        .bind(dto.work_date)
        .bind(&dto.hours)
        .bind(dto.activity_type.as_deref().unwrap_or("Direct"))
        .bind(dto.production_order_uuid)
        .bind(dto.operation_execution_uuid)
        .bind(dto.work_center_code.as_deref())
        .bind(dto.labour_category_uuid)
        .bind(dto.memo.as_deref())
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn bulk_submit_time_entries(&self, entry_uuids: &[Uuid]) -> Result<i32, AppError> {
        let n: (i32,) = sqlx::query_as("SELECT manufacturing.bulk_submit_time_entries($1)")
            .bind(entry_uuids)
            .fetch_one(&self.pool)
            .await?;
        Ok(n.0)
    }

    pub async fn bulk_approve_time_entries(
        &self,
        entry_uuids: &[Uuid],
        approver: Uuid,
        hourly_rate: Option<BigDecimal>,
    ) -> Result<i32, AppError> {
        let n: (i32,) = sqlx::query_as("SELECT manufacturing.bulk_approve_time_entries($1,$2,$3)")
            .bind(entry_uuids)
            .bind(approver)
            .bind(hourly_rate)
            .fetch_one(&self.pool)
            .await?;
        Ok(n.0)
    }

    pub async fn bulk_post_time_entries(
        &self,
        dto: BulkTimeEntryIdsDto,
        user_uuid: Uuid,
    ) -> Result<i32, AppError> {
        let n: (i32,) =
            sqlx::query_as("SELECT manufacturing.bulk_post_time_entries($1,$2,$3,$4,$5)")
                .bind(&dto.entry_uuids)
                .bind(user_uuid)
                .bind(dto.wip_account_code.as_deref().unwrap_or(""))
                .bind(dto.wages_account_code.as_deref().unwrap_or(""))
                .bind(dto.overhead_control_account.as_deref())
                .fetch_one(&self.pool)
                .await?;
        Ok(n.0)
    }

    pub async fn allocate_payrun_labour(
        &self,
        dto: AllocatePayrunLabourDto,
        user_uuid: Uuid,
    ) -> Result<Vec<PayrunLabourAllocationRow>, AppError> {
        let rows = sqlx::query_as::<_, PayrunLabourAllocationRow>(
            r#"
            SELECT * FROM manufacturing.allocate_payrun_labour_to_orders(
                $1,$2,$3,$4,$5
            )
            "#,
        )
        .bind(dto.payrun_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.labour_clearing_account.as_deref().unwrap_or(""))
        .bind(dto.cost_basis.as_deref().unwrap_or("employer_cost"))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn draft_time_from_order_ops(
        &self,
        order_uuid: Uuid,
        user_uuid: Uuid,
    ) -> Result<Vec<LabourTimeEntry>, AppError> {
        let rows = sqlx::query_as::<_, LabourTimeEntry>(
            "SELECT * FROM manufacturing.draft_time_entries_from_order_ops($1,$2)",
        )
        .bind(order_uuid)
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn upsert_attendance(
        &self,
        dto: UpsertAttendanceDto,
        user_uuid: Uuid,
    ) -> Result<AttendanceEntry, AppError> {
        let row = sqlx::query_as::<_, AttendanceEntry>(
            r#"
            SELECT * FROM hr.upsert_attendance(
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10
            )
            "#,
        )
        .bind(dto.employee_uuid)
        .bind(dto.work_date)
        .bind(dto.status.as_deref().unwrap_or("Present"))
        .bind(dto.clock_in)
        .bind(dto.clock_out)
        .bind(dto.break_minutes)
        .bind(dto.hours_present)
        .bind(dto.source.as_deref().unwrap_or("Manual"))
        .bind(dto.notes.as_deref())
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn labour_hours_reconciliation(
        &self,
        employee_uuid: Option<Uuid>,
    ) -> Result<Vec<LabourHoursReconciliation>, AppError> {
        let rows = if let Some(id) = employee_uuid {
            sqlx::query_as::<_, LabourHoursReconciliation>(
                "SELECT * FROM manufacturing.v_labour_hours_reconciliation WHERE employee_uuid = $1 ORDER BY work_date",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, LabourHoursReconciliation>(
                "SELECT * FROM manufacturing.v_labour_hours_reconciliation ORDER BY work_date DESC LIMIT 500",
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    pub async fn labour_hours_reconciliation_period(
        &self,
    ) -> Result<Vec<LabourHoursReconciliationPeriod>, AppError> {
        let rows = sqlx::query_as::<_, LabourHoursReconciliationPeriod>(
            "SELECT * FROM manufacturing.v_labour_hours_reconciliation_period ORDER BY month_start DESC, employee_uuid",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn allocate_payslip_labour(
        &self,
        dto: AllocatePayslipLabourDto,
        user_uuid: Uuid,
    ) -> Result<Vec<PayslipLabourAllocationRow>, AppError> {
        let rows = sqlx::query_as::<_, PayslipLabourAllocationRow>(
            r#"
            SELECT * FROM manufacturing.allocate_payslip_labour_to_orders(
                $1, $2, $3, $4, $5
            )
            "#,
        )
        .bind(dto.payslip_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.labour_clearing_account.as_deref().unwrap_or(""))
        .bind(dto.cost_basis.as_deref().unwrap_or("employer_cost"))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn labour_productivity_by_order(
        &self,
        order_uuid: Option<Uuid>,
    ) -> Result<Vec<LabourProductivityByOrder>, AppError> {
        let rows = if let Some(id) = order_uuid {
            sqlx::query_as::<_, LabourProductivityByOrder>(
                "SELECT * FROM manufacturing.v_labour_productivity_by_order WHERE production_order_uuid = $1",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, LabourProductivityByOrder>(
                "SELECT * FROM manufacturing.v_labour_productivity_by_order ORDER BY order_number",
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    pub async fn labour_by_employee(&self) -> Result<Vec<LabourByEmployee>, AppError> {
        let rows = sqlx::query_as::<_, LabourByEmployee>(
            "SELECT * FROM manufacturing.v_labour_by_employee ORDER BY month_start DESC, employee_uuid",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn labour_by_work_center(&self) -> Result<Vec<LabourByWorkCenter>, AppError> {
        let rows = sqlx::query_as::<_, LabourByWorkCenter>(
            "SELECT * FROM manufacturing.v_labour_by_work_center ORDER BY month_start DESC, work_center_code",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn operation_efficiency(
        &self,
        order_uuid: Option<Uuid>,
    ) -> Result<Vec<OperationEfficiency>, AppError> {
        let rows = if let Some(id) = order_uuid {
            sqlx::query_as::<_, OperationEfficiency>(
                "SELECT * FROM manufacturing.v_operation_efficiency WHERE production_order_uuid = $1",
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, OperationEfficiency>(
                "SELECT * FROM manufacturing.v_operation_efficiency ORDER BY order_number, sequence",
            )
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows)
    }

    pub async fn compute_labour_variances(
        &self,
        order_uuid: Uuid,
        as_of: Option<chrono::NaiveDate>,
    ) -> Result<LabourVariance, AppError> {
        let row = sqlx::query_as::<_, LabourVariance>(
            "SELECT * FROM manufacturing.compute_labour_variances($1, COALESCE($2, CURRENT_DATE))",
        )
        .bind(order_uuid)
        .bind(as_of)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn production_order_labour_summary(
        &self,
        order_uuid: Uuid,
    ) -> Result<ProductionOrderLabourSummary, AppError> {
        let row = sqlx::query_as::<_, ProductionOrderLabourSummary>(
            r#"
            SELECT * FROM manufacturing.v_production_order_labour
            WHERE production_order_uuid = $1
            "#,
        )
        .bind(order_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_labour_allocation(
        &self,
        dto: CreateLabourAllocationDto,
        user_uuid: Uuid,
    ) -> Result<LabourAllocation, AppError> {
        let row = sqlx::query_as::<_, LabourAllocation>(
            r#"
            SELECT * FROM manufacturing.create_labour_allocation(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            "#,
        )
        .bind(dto.production_order_uuid)
        .bind(&dto.labour_class)
        .bind(&dto.hours)
        .bind(&dto.rate)
        .bind(dto.work_center_code.as_deref())
        .bind(dto.employee_uuid)
        .bind(dto.source.as_deref().unwrap_or("Manual"))
        .bind(dto.source_ref.as_deref())
        .bind(dto.memo.as_deref())
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn post_labour_allocation(
        &self,
        dto: PostLabourAllocationDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let row = sqlx::query_as::<_, CostApplication>(
            r#"
            SELECT * FROM manufacturing.post_labour_allocation(
                $1, $2, $3, $4, $5
            )
            "#,
        )
        .bind(dto.allocation_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.wages_account_code.as_deref().unwrap_or(""))
        .bind(dto.overhead_control_account.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn ensure_operation_executions(&self, order_uuid: Uuid) -> Result<i32, AppError> {
        let n: i32 = sqlx::query_scalar("SELECT manufacturing.ensure_operation_executions($1)")
            .bind(order_uuid)
            .fetch_one(&self.pool)
            .await?;
        Ok(n)
    }

    pub async fn list_operation_executions(
        &self,
        order_uuid: Uuid,
    ) -> Result<Vec<OperationExecution>, AppError> {
        let rows = sqlx::query_as::<_, OperationExecution>(
            r#"
            SELECT * FROM manufacturing.operation_executions
            WHERE production_order_uuid = $1
            ORDER BY sequence NULLS LAST, created_at
            "#,
        )
        .bind(order_uuid)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn start_operation(
        &self,
        dto: StartOperationDto,
    ) -> Result<OperationExecution, AppError> {
        let row = sqlx::query_as::<_, OperationExecution>(
            "SELECT * FROM manufacturing.start_operation($1, $2, $3)",
        )
        .bind(dto.execution_uuid)
        .bind(dto.qty_started)
        .bind(dto.operator_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn complete_operation(
        &self,
        dto: CompleteOperationDto,
    ) -> Result<OperationExecution, AppError> {
        let row = sqlx::query_as::<_, OperationExecution>(
            "SELECT * FROM manufacturing.complete_operation($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(dto.execution_uuid)
        .bind(&dto.qty_completed)
        .bind(dto.qty_scrapped)
        .bind(dto.setup_minutes_actual)
        .bind(dto.run_minutes_actual)
        .bind(dto.machine_minutes_actual)
        .bind(dto.operator_uuid)
        .bind(dto.notes.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn record_scrap_rework(
        &self,
        dto: RecordScrapReworkDto,
        user_uuid: Uuid,
    ) -> Result<ScrapReworkEvent, AppError> {
        let row = sqlx::query_as::<_, ScrapReworkEvent>(
            "SELECT * FROM manufacturing.record_scrap_rework($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(dto.production_order_uuid)
        .bind(&dto.event_type)
        .bind(&dto.quantity)
        .bind(dto.operation_execution_uuid)
        .bind(dto.reason_code.as_deref())
        .bind(dto.reason_text.as_deref())
        .bind(dto.disposition.as_deref())
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn record_quality_inspection(
        &self,
        dto: RecordQualityInspectionDto,
        user_uuid: Uuid,
    ) -> Result<QualityInspection, AppError> {
        let row = sqlx::query_as::<_, QualityInspection>(
            "SELECT * FROM manufacturing.record_quality_inspection($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(dto.production_order_uuid)
        .bind(&dto.qty_inspected)
        .bind(&dto.qty_passed)
        .bind(&dto.qty_failed)
        .bind(&dto.result)
        .bind(dto.inspection_type.as_deref().unwrap_or("InProcess"))
        .bind(dto.operation_execution_uuid)
        .bind(user_uuid)
        .bind(dto.notes.as_deref())
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn apply_direct_labor(
        &self,
        dto: ApplyDirectLaborDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let cost_application: CostApplication = sqlx::query_as::<_, CostApplication>(
            "
                SELECT * FROM manufacturing.apply_direct_labor_from_routing(
                    $1, $2, $3, $4, $5
                )
            ",
        )
        .bind(dto.production_order_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.labor_account_code.as_deref().unwrap_or(""))
        .bind(dto.reference)
        .fetch_one(&self.pool)
        .await?;

        Ok(cost_application)
    }

    pub async fn calculate_standard_cost_for_item(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item_uuid: Uuid,
    ) -> Result<Cost, AppError> {
        let standard_cost = sqlx::query_as::<_, Cost>(
            "SELECT manufacturing.calculate_standard_cost($1) AS proration",
        )
        .bind(item_uuid)
        .fetch_one(&mut **tx)
        .await?;

        Ok(standard_cost)
    }

    // ============================ Complete Production Order ============================
    /*pub async fn complete_production_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        let _ = sqlx::query_as::<_, ProductionOrder>(
            "SELECT * FROM manufacturing.complete_production_order(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_DATE
            ) as gl_txn_uuid",
        )
        .bind(dto.production_order_uuid)
        .bind(dto.completed_quantity)
        .bind(user_uuid)
        .bind(dto.control_code.as_deref().unwrap_or(""))
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.fg_account_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_mat_var_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_lab_var_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_moh_var_code.as_deref().unwrap_or(""))
        .fetch_one(&self.pool)
        .await?;

        // Refresh the order object
        self.get_production_order(dto.production_order_uuid).await
    }*/

    /// Planned → In Progress. Required before complete_production_order.
    pub async fn start_production_order(
        &self,
        production_order_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        let order: Option<ProductionOrder> = sqlx::query_as(
            r#"
            UPDATE manufacturing.production_orders
            SET status = 'In Progress',
                updated_at = now()
            WHERE uuid = $1
              AND status IN ('Planned', 'In Progress')
            RETURNING *
            "#,
        )
        .bind(production_order_uuid)
        .fetch_optional(&self.pool)
        .await?;

        match order {
            Some(o) => Ok(o),
            None => {
                // Distinguish not-found vs wrong status
                let existing: Option<(String,)> = sqlx::query_as(
                    "SELECT status FROM manufacturing.production_orders WHERE uuid = $1",
                )
                .bind(production_order_uuid)
                .fetch_optional(&self.pool)
                .await?;
                match existing {
                    None => Err(AppError::NotFound("Production order not found".into())),
                    Some((status,)) => Err(AppError::Unprocessable(format!(
                        "Cannot start production order in status '{status}' (expected Planned or In Progress)"
                    ))),
                }
            }
        }
    }

    pub async fn complete_production_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        let _gl_txn: Uuid = sqlx::query_scalar(
            r#"
        SELECT manufacturing.complete_production_order(
            $1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_DATE, $10
        )
        "#,
        )
        .bind(dto.production_order_uuid)
        .bind(dto.completed_quantity)
        .bind(user_uuid)
        .bind(dto.control_code.as_deref().unwrap_or(""))
        .bind(dto.wip_account_code.as_deref().unwrap_or(""))
        .bind(dto.fg_account_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_mat_var_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_lab_var_code.as_deref().unwrap_or(""))
        .bind(dto.mfg_moh_var_code.as_deref().unwrap_or(""))
        .bind(dto.fg_warehouse_serial)
        .fetch_one(&self.pool)
        .await?;

        self.get_production_order(dto.production_order_uuid).await
    }

    // ============================ Manual Variance Proration v2.0 ============================
    pub async fn prorate_variance(
        &self,
        dto: ProrateVarianceDto,
        user_uuid: Uuid,
    ) -> Result<VarianceProrationResult, AppError> {
        let as_of: NaiveDate = dto
            .as_of_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let _ = sqlx::query(
            r#"
            CALL manufacturing.prorate_variance(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            "#,
        )
        .bind(dto.variance_amount)
        .bind(user_uuid)
        .bind(dto.wip_account.as_deref().unwrap_or(""))
        .bind(dto.fg_account)
        .bind(dto.cogs_account)
        .bind(as_of)
        .bind(dto.min_allocation_threshold.unwrap_or(BigDecimal::from(5)))
        .bind(dto.materiality_threshold.unwrap_or(BigDecimal::from(100)))
        .bind(
            dto.memo
                .unwrap_or_else(|| "Manual overhead variance proration".to_string()),
        )
        .bind(dto.dry_run.unwrap_or(false))
        .execute(&self.pool)
        .await?;

        // After CALL we fetch the log to return full result
        let proration: VarianceProrationResult = sqlx::query_as::<_, VarianceProrationResult>(
            r#"
            SELECT
                gl_transaction_uuid,
                wip_alloc,
                fg_alloc,
                cogs_alloc,
                (wip_alloc + fg_alloc + cogs_alloc) as total_allocated
            FROM accounting.variance_proration_logs
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(proration)
    }

    pub async fn create_header(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        dto: &CreateBomHeaderDto,
        user_uuid: Uuid,
    ) -> Result<BomHeader, AppError> {
        let bom_header: BomHeader = sqlx::query_as::<_, BomHeader>(
            r#"
            INSERT INTO manufacturing.bom_headers (
                bom_code, product_item_uuid, description, revision, is_active,
                is_default, created_by
            ) VALUES (
                $1, $2, $3, COALESCE($4, 'A'), COALESCE($5, true),
                COALESCE($6, false), $7
            ) RETURNING *
            "#,
        )
        .bind(&dto.bom_code)
        .bind(dto.product_item_uuid)
        .bind(&dto.description)
        .bind(dto.revision.as_deref())
        .bind(dto.is_active.unwrap_or(true))
        .bind(dto.is_default.unwrap_or(false))
        .bind(user_uuid)
        .fetch_one(&mut **tx)
        .await?;

        Ok(bom_header)
    }

    pub async fn get_full_bom(&self, bom_uuid: Uuid) -> Result<Option<BomWithLines>, AppError> {
        let header_opt: Option<BomHeader> = sqlx::query_as::<_, BomHeader>(
            "SELECT * FROM manufacturing.bom_headers WHERE uuid = $1",
        )
        .bind(bom_uuid)
        .fetch_optional(&self.pool)
        .await?;

        let Some(header) = header_opt else {
            return Ok(None);
        };

        let lines: Vec<BomLine> = sqlx::query_as::<_, BomLine>(
            "SELECT * FROM manufacturing.bom_lines WHERE bom_header_uuid = $1 ORDER BY line_number",
        )
        .bind(bom_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(BomWithLines { header, lines }))
    }

    /// Get default/active BOM for a product
    pub async fn get_default_bom_for_product(
        &self,
        product_uuid: Uuid,
    ) -> Result<Option<BomWithLines>, AppError> {
        let header_opt: Option<BomHeader> = sqlx::query_as::<_, BomHeader>(
            r#"
            SELECT * FROM manufacturing.bom_headers
            WHERE product_item_uuid = $1
              AND is_active
            ORDER BY is_default DESC, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(product_uuid)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(header) = header_opt {
            let lines: Vec<BomLine> = sqlx::query_as::<_, BomLine>(
                "SELECT * FROM manufacturing.bom_lines WHERE bom_header_uuid = $1 ORDER BY line_number"
            )
            .bind(header.uuid)
            .fetch_all(&self.pool)
            .await?;

            Ok(Some(BomWithLines { header, lines }))
        } else {
            Ok(None)
        }
    }

    // ============================ Costing policy (Stage A) ============================
    pub async fn load_costing_policy(&self) -> Result<CostingPolicy, AppError> {
        let row: Option<CostingPolicy> = sqlx::query_as(
            r#"
            SELECT uuid, costing_method, oh_application_base,
                   use_timesheet_actual, use_payroll_allocation,
                   effective_from, notes, updated_at, updated_by
            FROM manufacturing.costing_policies
            ORDER BY effective_from DESC, updated_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(p) = row {
            return Ok(p);
        }

        // Self-heal if seed row missing (legacy tenants)
        sqlx::query(
            r#"
            INSERT INTO manufacturing.costing_policies (
                costing_method, oh_application_base,
                use_timesheet_actual, use_payroll_allocation
            ) VALUES ('standard', 'labor_hours', true, true)
            "#,
        )
        .execute(&self.pool)
        .await?;

        let p: CostingPolicy = sqlx::query_as(
            r#"
            SELECT uuid, costing_method, oh_application_base,
                   use_timesheet_actual, use_payroll_allocation,
                   effective_from, notes, updated_at, updated_by
            FROM manufacturing.costing_policies
            ORDER BY effective_from DESC, updated_at DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(p)
    }

    pub async fn update_costing_policy(
        &self,
        dto: UpdateCostingPolicyDto,
        updated_by: Uuid,
    ) -> Result<CostingPolicy, AppError> {
        let current = self.load_costing_policy().await?;

        let method = dto
            .costing_method
            .as_deref()
            .map(|s| {
                CostingMethod::parse(s).ok_or_else(|| {
                    AppError::BadRequest(
                        "costing_method must be actual | normal | standard".into(),
                    )
                })
            })
            .transpose()?
            .unwrap_or_else(|| current.method());

        let oh_base = dto
            .oh_application_base
            .as_deref()
            .map(|s| {
                OhApplicationBase::parse(s).ok_or_else(|| {
                    AppError::BadRequest(
                        "oh_application_base must be labor_hours | machine_hours | units".into(),
                    )
                })
            })
            .transpose()?
            .unwrap_or_else(|| current.oh_base());

        let use_ts = dto
            .use_timesheet_actual
            .unwrap_or(current.use_timesheet_actual);
        let use_pr = dto
            .use_payroll_allocation
            .unwrap_or(current.use_payroll_allocation);
        if !use_ts && !use_pr {
            return Err(AppError::BadRequest(
                "At least one of use_timesheet_actual or use_payroll_allocation must be true".into(),
            ));
        }

        let effective_from = dto.effective_from.unwrap_or(current.effective_from);
        let notes = dto.notes.or(current.notes);

        let p: CostingPolicy = sqlx::query_as(
            r#"
            UPDATE manufacturing.costing_policies SET
                costing_method = $2,
                oh_application_base = $3,
                use_timesheet_actual = $4,
                use_payroll_allocation = $5,
                effective_from = $6,
                notes = $7,
                updated_at = now(),
                updated_by = $8
            WHERE uuid = $1
            RETURNING uuid, costing_method, oh_application_base,
                      use_timesheet_actual, use_payroll_allocation,
                      effective_from, notes, updated_at, updated_by
            "#,
        )
        .bind(current.uuid)
        .bind(method.as_str())
        .bind(oh_base.as_str())
        .bind(use_ts)
        .bind(use_pr)
        .bind(effective_from)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(p)
    }

}
