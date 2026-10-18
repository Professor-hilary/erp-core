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
        let order = sqlx::query_as::<_, ProductionOrder>(
            r#"
            INSERT INTO manufacturing.production_orders(
                order_number, product_item_uuid, quantity_ordered, start_date,
                expected_completion_date, status
            ) VALUES ($1, $2, $3, $4, $5, 'Planned')
            RETURNING *
            "#,
        )
        .bind(&dto.order_number)
        .bind(dto.product_item_uuid)
        .bind(&dto.quantity_ordered)
        .bind(dto.start_date)
        .bind(dto.expected_completion_date)
        .fetch_one(&self.pool)
        .await?;

        // Multi-level: explode default BOM to leaves and aggregate quantities
        sqlx::query(
            r#"
            INSERT INTO manufacturing.production_order_materials(
                production_order_uuid, bom_line_uuid, component_item_uuid, required_qty
            )
            SELECT
                p.uuid,
                NULL,                          -- no single top-level line after explosion
                e.component_item_uuid,
                SUM(e.quantity_required)
            FROM manufacturing.production_orders p
            JOIN manufacturing.bom_headers bh
              ON bh.product_item_uuid = p.product_item_uuid
             AND bh.is_active
             AND bh.is_default = true
            CROSS JOIN LATERAL manufacturing.explode_bom(bh.uuid, p.quantity_ordered) e
            WHERE p.uuid = $1
              AND e.is_leaf = true
            GROUP BY p.uuid, e.component_item_uuid
            "#,
        )
        .bind(order.uuid)
        .execute(&self.pool)
        .await?;

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
        .bind(&dto.raw_mat_account_code)
        .bind(dto.wip_account_code)
        .bind(user_uuid)
        .fetch_all(&self.pool)
        .await?;

        Ok(materials)
    }

    pub async fn issue_material_to_prod(
        &self,
        dto: SingleMaterialIssueDto,
        user_uuid: Uuid,
    ) -> Result<Vec<FullMaterialIssue>, AppError> {
        let materials: Vec<FullMaterialIssue> = sqlx::query_as::<_, FullMaterialIssue>(
            r#"SELECT * FROM manufacturing.issue_material_to_order($1, $2, $3, $4, $5, $6, $7)"#,
        )
        .bind(dto.item_serial_id)
        .bind(dto.warehouse_serial)
        .bind(dto.quantity)
        .bind(&dto.production_order_uuid)
        .bind(&dto.raw_materials_code)
        .bind(dto.work_in_progress_code)
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
        .bind(dto.wip_account)
        .bind(dto.overhead_control_account)
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
        .bind(dto.payable_or_cash)
        .bind(dto.overhead_control_account)
        .bind(dto.allocation_base)
        .bind(dto.reference)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(cost_application)
    }

	// =============================== Labor Application ============================
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
        .bind(dto.wip_account_code)
        .bind(dto.labor_account_code)
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
    pub async fn complete_production_order(
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
        .bind(dto.control_code)
        .bind(dto.wip_account_code)
        .bind(dto.fg_account_code)
        .bind(dto.mfg_mat_var_code)
        .bind(dto.mfg_lab_var_code)
        .bind(dto.mfg_moh_var_code)
        .fetch_one(&self.pool)
        .await?;

        // Refresh the order object
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
        .bind(dto.wip_account)
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
}
