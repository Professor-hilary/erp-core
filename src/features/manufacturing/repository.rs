// src/features/accounts/repositories.rs
use crate::models::manufacturing::*;
use bigdecimal::{BigDecimal, One, Zero};
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
    ) -> Result<ProductionOrder, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO manufacturing.production_orders(
                order_number, product_item_id, quantity_ordered, start_date,
                expected_completion_date, status
            ) VALUES ($1, $2, $3, $4, $5, 'Planned')
            RETURNING *
            "#,
        )
        .bind(&dto.order_number)
        .bind(dto.product_item_serial_id)
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
            product_item_serial_id: row.get("product_item_id"),
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
        sqlx::query("SELECT manufacturing.intitlize_material_cost($1)")
            .bind(order.uuid)
            .execute(&self.pool)
            .await?;

        Ok(order)
    }

    pub async fn create_overhead_rate(
        &self,
        dto: CreateOverheadRateDto,
    ) -> Result<OverheadRates, sqlx::Error> {
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
        .bind(dto.department_code)
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

    pub async fn get_production_order(&self, uuid: Uuid) -> Result<ProductionOrder, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM manufacturing.production_orders WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(ProductionOrder {
            uuid: row.get("uuid"),
            serial_id: row.get("serial_id"),
            order_number: row.get("order_number"),
            product_item_serial_id: row.get("product_item_id"),
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
    ) -> Result<MaterialIssue, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM inventory.deplete_inventory(
                $1, $2, $3, 'PRODUCTION',
                (SELECT serial_id FROM manufacturing.production_orders WHERE uuid = $4),
                $5, NULL
            ) as total_cost
            "#,
        )
        .bind(dto.item_serial_id)
        .bind(dto.warehouse_serial_id)
        .bind(&dto.quantity)
        .bind(dto.production_order_uuid)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        let total_cost: BigDecimal = row.get("total_cost");
        let quantity: &BigDecimal = &dto.quantity;

        // Division by protection and rounding to 2 dp if division returns more points
        let unit_cost: BigDecimal = if quantity.is_zero() {
            BigDecimal::from(0)
        } else {
            (&total_cost / quantity).with_scale(4)
        };

        // Insert into material_issues table
        let issue_row: PgRow = sqlx::query(
            r#"
            INSERT INTO manufacturing.material_issues(
                production_order_uuid, item_serial_id, warehouse_serial_id, quantity,
                unit_cost
            ) VALUES ($1, $2, $3, $4, $5) RETURNING *
            "#,
        )
        .bind(dto.production_order_uuid)
        .bind(dto.item_serial_id)
        .bind(dto.warehouse_serial_id)
        .bind(dto.quantity)
        .bind(unit_cost)
        .fetch_one(&self.pool)
        .await?;

        Ok(MaterialIssue {
            uuid: issue_row.get("uuid"),
            production_order_uuid: issue_row.get("production_order_uuid"),
            item_serial_id: issue_row.get("item_serial_id"),
            warehouse_serial_id: issue_row.get("warehouse_serial_id"),
            quantity: issue_row.get("quantity"),
            total_cost: issue_row.get("total_cost"),
            issued_at: issue_row.get("issued_at"),
        })
    }

    // ============== Overhead Application ==============
    pub async fn apply_overhead(
        &self,
        dto: ApplyOverheadDto,
        user_uuid: Uuid,
    ) -> Result<CostApplication, sqlx::Error> {
        let row = sqlx::query("SELECT manufacturing.apply_overhead($1, $2, $3) as applied_amount")
            .bind(dto.production_order_uuid)
            .bind(dto.base_amount)
            .bind(user_uuid)
            .fetch_one(&self.pool)
            .await?;

        let amount: BigDecimal = row.get("applied_amount");

        let app_row = sqlx::query(
            r#"
            INSERT INTO manufacturing.cost_applications
                (production_order_uuid, type, amount)
            VALUES ($1, 'Overhead', $2)
            RETURNING *
            "#,
        )
        .bind(dto.production_order_uuid)
        .bind(amount)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostApplication {
            uuid: app_row.get("uuid"),
            production_order_uuid: app_row.get("production_order_uuid"),
            application_type: "Overhead".to_string(),
            amount: app_row.get("amount"),
            applied_at: app_row.get("applied_at"),
            reference: app_row.get("reference"),
        })
    }

    // ============== Complete Production Order ==============
    pub async fn complete_production_order(
        &self,
        dto: CompleteProductionOrderDto,
        user_uuid: Uuid,
    ) -> Result<ProductionOrder, sqlx::Error> {
        let _ = sqlx::query(
            "SELECT * FROM manufacturing.complete_production_order($1, $2, CURRENT_DATE, $3) as gl_txn_uuid"
        )
        .bind(dto.production_order_uuid)
        .bind(dto.completed_quantity)
        .bind(user_uuid)
        .fetch_one(&self.pool)
        .await?;

        // Refresh the order object
        self.get_production_order(dto.production_order_uuid).await
    }

    // ============== Manual Variance Proration v2.0 ==============
    pub async fn prorate_variance_v2(
        &self,
        dto: ProrateVarianceDto,
        user_uuid: Uuid,
    ) -> Result<VarianceProrationResult, sqlx::Error> {
        let as_of = dto
            .as_of_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let _ = sqlx::query(
            r#"
            CALL manufacturing.prorate_variance_v2(
                $1, $2, 5.00, 100.00, $3, $4, $5
            )
            "#,
        )
        .bind(dto.variance_amount)
        .bind(as_of)
        .bind(
            dto.memo
                .unwrap_or_else(|| "Manual overhead variance proration".to_string()),
        )
        .bind(user_uuid)
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
        dto: CreateBomHeaderDto,
        user_uuid: Uuid,
    ) -> Result<BomHeader, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO manufacturing.bom_headers (
                bom_code, product_item_uuid, description, revision, is_active,
                is_default, created_by
            ) VALUES (
                $1, $2, $3, COALESCE($4, 'A'), COALESCE($5, true),
                COALESCE($5, false), $6
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
    ) -> Result<BomLine, sqlx::Error> {
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

    pub async fn get_full_bom(&self, bom_uuid: Uuid) -> Result<Option<BomWithLines>, sqlx::Error> {
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
    ) -> Result<Option<BomWithLines>, sqlx::Error> {
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
