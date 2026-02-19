// src/features/accounts/repositories.rs
use crate::models::manufacturing::*;
use bigdecimal::BigDecimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct ManufacturingRepo {
    pool: PgPool,
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
            INSERT INTO manufacturing.production_orders
                (order_number, product_item_id, quantity_ordered, start_date, expected_completion_date, status)
            VALUES ($1, $2, $3, $4, $5, 'Planned')
            RETURNING *
            "#
        )
        .bind(&dto.order_number)
        .bind(dto.product_item_serial_id)
        .bind(dto.quantity_ordered)
        .bind(dto.start_date)
        .bind(dto.expected_completion_date)
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

    pub async fn get_production_order(&self, uuid: Uuid) -> Result<ProductionOrder, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM manufacturing.production_orders WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        // same mapping as above...
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
        }) // I'll keep it short here – copy pattern from create
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

        // Insert into material_issues table
        let issue_row = sqlx::query(
            r#"
            INSERT INTO manufacturing.material_issues
                (production_order_uuid, item_serial_id, warehouse_serial_id, quantity, total_cost)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(dto.production_order_uuid)
        .bind(dto.item_serial_id)
        .bind(dto.warehouse_serial_id)
        .bind(dto.quantity)
        .bind(total_cost)
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
}
