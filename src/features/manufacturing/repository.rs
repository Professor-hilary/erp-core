// src/features/accounts/repositories.rs
use crate::{interface::api::errors::AppError, models::manufacturing::*};
use bigdecimal::{BigDecimal, One};
use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;
pub struct ManufacturingRepo {
    pub(crate) pool: PgPool,
}

impl ManufacturingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============== Production Order ==============
    pub async fn create_production_order(
        &self,
        dto: CreateProductionOrderDto,
    ) -> Result<ProductionOrder, AppError> {
        let row = sqlx::query(
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
        .bind(dto.quantity_ordered)
        .bind(dto.start_date)
        .bind(dto.expected_completion_date)
        .fetch_one(&self.pool)
        .await?;

        // Update cost tracker to record material cost
        let order: ProductionOrder = ProductionOrder {
            uuid: row.get("uuid"),
            serial_id: row.get("serial_id"),
            order_number: row.get("order_number"),
            product_item_uuid: row.get("product_item_uuid"),
            quantity_ordered: row.get("quantity_ordered"),
            quantity_completed: row.get("quantity_completed"),
            status: row.get("status"),
            start_date: row.get("start_date"),
            expected_completion_date: row.get("expected_completion_date"),
            actual_completion_date: row.get("actual_completion_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        // Update the cost overhead table for end of period overhead adjustment
        // sqlx::query("SELECT manufacturing.initialize_material_cost($1)")
        //     .bind(order.uuid)
        //     .execute(&self.pool)
        //     .await?;

        Ok(order)
    }

    pub async fn create_overhead_rate(
        &self,
        dto: CreateOverheadRateDto,
    ) -> Result<OverheadRates, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO manufacturing.overhead_rates
                (period_start, period_end, allocation_base, estimated_overhead,
                estimated_base, department_code, is_active)
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
        Ok(OverheadRates {
            uuid: row.get("uuid"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            allocation_base: row.get("allocation_base"),
            estimated_base: row.get("estimated_base"),
            estimated_overhead: row.get("estimated_overhead"),
            rate: row.get("rate"),
            department_code: row.get("department_code"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_production_order(&self, uuid: Uuid) -> Result<ProductionOrder, AppError> {
        let row = sqlx::query("SELECT * FROM manufacturing.production_orders WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(ProductionOrder {
            uuid: row.get("uuid"),
            serial_id: row.get("serial_id"),
            order_number: row.get("order_number"),
            product_item_uuid: row.get("product_item_uuid"),
            quantity_ordered: row.get("quantity_ordered"),
            quantity_completed: row.get("quantity_completed"),
            status: row.get("status"),
            start_date: row.get("start_date"),
            expected_completion_date: row.get("expected_completion_date"),
            actual_completion_date: row.get("actual_completion_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    // ============== Material Issue ==============
    pub async fn issue_material(
        &self,
        dto: IssueMaterialDto,
        user_uuid: Uuid,
    ) -> Result<MaterialIssue, AppError> {
        let row = sqlx::query_as::<_, MaterialIssue>(
            r#"SELECT * FROM manufacturing.issue_material_to_order($1, $2, $3, $4, $5)"#,
        )
        .bind(dto.stock_item_id)
        .bind(dto.warehouse_serial_id)
        .bind(&dto.quantity)
        .bind(dto.production_order_uuid)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    // ============== Overhead Application ==============
    pub async fn apply_overhead_to_order(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let row: PgRow = sqlx::query(
            "
            SELECT * FROM manufacturing.apply_overhead_to_order(
                $1, $2, $3, $4, $5
            )
        ",
        )
        .bind(dto.production_order_uuid)
        .bind(dto.base_amount)
        .bind(dto.wip_account)
        .bind(dto.overhead_control_account)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostApplication {
            uuid: row.get("uuid"),
            production_order_uuid: row.get("production_order_uuid"),
            application_type: "Overhead".to_string(),
            amount: row.get("amount"),
            applied_at: row.get("applied_at"),
            reference: row.get("reference"),
        })
    }

    // ============== Recognize Overhead Expenditure ==============
    pub async fn recognize_overhead(
        &self,
        dto: RecognizeOverhead,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let row: PgRow = sqlx::query(
            "
            SELECT * FROM manufacturing.apply_overhead_to_control_account(
                $1, $2, $3, $4, $5, $6
            )
        ",
        )
        .bind(dto.production_order_uuid)
        .bind(dto.base_amount)
        .bind(dto.payable_or_cash)
        .bind(dto.overhead_control_account)
        .bind(dto.allocation_base)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostApplication {
            uuid: row.get("uuid"),
            production_order_uuid: row.get("production_order_uuid"),
            application_type: "Overhead".to_string(),
            amount: row.get("amount"),
            applied_at: row.get("applied_at"),
            reference: row.get("reference"),
        })
    }

    // ============== Labor Application ==============
    pub async fn apply_labor_costs(
        &self,
        dto: ApplyLaborCostDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, AppError> {
        let row: PgRow = sqlx::query(
            "
            SELECT * FROM manufacturing.apply_labor_cost(
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
        ",
        )
        .bind(dto.production_order_uuid)
        .bind(dto.hours)
        .bind(dto.rate_per_hour)
        .bind(dto.is_direct)
        .bind(user_uuid)
        .bind(dto.renumeration_code)
        .bind(dto.control_account_code)
        .bind(dto.wip_account_code)
        .bind(dto.department_code.unwrap_or_default())
        .bind(dto.reference)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostApplication {
            uuid: row.get("uuid"),
            production_order_uuid: row.get("production_order_uuid"),
            application_type: "Overhead".to_string(),
            amount: row.get("amount"),
            applied_at: row.get("applied_at"),
            reference: row.get("reference"),
        })
    }

    pub async fn calculate_standard_cost_for_item(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        item_uuid: Uuid,
    ) -> Result<BigDecimal, AppError> {
        let row = sqlx::query("SELECT manufacturing.calculate_standard_cost($1) AS standard_cost")
            .bind(item_uuid)
            .fetch_one(&mut **tx)
            .await?;

        Ok(row.get("standard_cost"))
    }

    // ============== Complete Production Order ==============
    pub async fn complete_production_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, AppError> {
        let _ = sqlx::query(
            "SELECT * FROM manufacturing.complete_production_order(
                $1, $2, $3, $4, $5, $6, $7, $8, CURRENT_DATE
            ) as gl_txn_uuid",
        )
        .bind(dto.production_order_uuid)
        .bind(user_uuid)
        .bind(dto.wip_account_code)
        .bind(dto.fg_account_code)
        .bind(dto.mfg_mat_var_code)
        .bind(dto.mfg_lab_var_code)
        .bind(dto.mfg_ovh_var_code)
        .bind(dto.completed_quantity)
        .fetch_one(&self.pool)
        .await?;

        // Refresh the order object
        self.get_production_order(dto.production_order_uuid).await
    }

    // ============== Manual Variance Proration v2.0 ==============
    pub async fn prorate_variance(
        &self,
        dto: ProrateVarianceDto,
        user_uuid: Uuid,
    ) -> Result<VarianceProrationResult, AppError> {
        let as_of = dto
            .as_of_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let _ = sqlx::query(
            r#"
            CALL manufacturing.prorate_variance(
                $1, $2, $3, $4, $5, $6, 5.00, 100.00, $7, $8
            )
            "#,
        )
        .bind(dto.variance_amount)
        .bind(user_uuid)
        .bind(dto.wip_account)
        .bind(dto.fg_account)
        .bind(dto.cogs_account)
        .bind(as_of)
        .bind(
            dto.memo
                .unwrap_or_else(|| "Manual overhead variance proration".to_string()),
        )
        .bind(dto.dry_run.unwrap_or(false))
        .execute(&self.pool)
        .await?;

        // After CALL we fetch the log to return full result
        let log_row = sqlx::query(
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

        Ok(VarianceProrationResult {
            gl_transaction_uuid: log_row.get("gl_transaction_uuid"),
            wip_alloc: log_row.get("wip_alloc"),
            fg_alloc: log_row.get("fg_alloc"),
            cogs_alloc: log_row.get("cogs_alloc"),
            total_allocated: log_row.get("total_allocated"),
        })
    }

    pub async fn create_header(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        dto: &CreateBomHeaderDto,
        user_uuid: Uuid,
    ) -> Result<BomHeader, AppError> {
        let row = sqlx::query(
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

        Ok(BomHeader {
            uuid: row.get("uuid"),
            serial_id: row.get("serial_id"),
            bom_code: row.get("bom_code"),
            product_item_uuid: row.get("product_item_uuid"),
            description: row.get("description"),
            revision: row.get("revision"),
            is_active: row.get("is_active"),
            is_default: row.get("is_default"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
        })
    }

    pub async fn add_line(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        header_uuid: Uuid,
        dto: &CreateBomLineDto,
    ) -> Result<BomLine, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO manufacturing.bom_lines (
                bom_header_uuid, line_number, component_item_uuid,
                quantity_per, uom, scrap_factor, notes
            ) VALUES ($1, $2, $3, $4, COALESCE($5, 'pcs'), COALESCE($6, 1.0000), $7)
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
        .fetch_one(&mut **tx)
        .await?;

        Ok(BomLine {
            uuid: row.get("uuid"),
            bom_header_uuid: row.get("bom_header_uuid"),
            line_number: row.get("line_number"),
            component_item_uuid: row.get("component_item_uuid"),
            quantity_per: row.get("quantity_per"),
            uom: row.get("uom"),
            scrap_factor: row.get("scrap_factor"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_full_bom(&self, bom_uuid: Uuid) -> Result<Option<BomWithLines>, AppError> {
        let header_opt = sqlx::query_as::<_, BomHeader>(
            "SELECT * FROM manufacturing.bom_headers WHERE uuid = $1",
        )
        .bind(bom_uuid)
        .fetch_optional(&self.pool)
        .await?;

        let Some(header) = header_opt else {
            return Ok(None);
        };

        let lines = sqlx::query_as::<_, BomLine>(
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
        let header_opt = sqlx::query_as::<_, BomHeader>(
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
            let lines = sqlx::query_as::<_, BomLine>(
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
