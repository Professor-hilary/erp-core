use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub order_number: String,
    pub product_item_serial_id: i64,        // links to inventory.items.serial_id
    pub quantity_ordered: BigDecimal,
    pub quantity_completed: BigDecimal,
    pub status: String,                     // "Planned" | "In Progress" | "Completed" | "Cancelled"
    pub start_date: Option<NaiveDate>,
    pub expected_completion_date: Option<NaiveDate>,
    pub actual_completion_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MaterialIssue {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub item_serial_id: i64,
    pub warehouse_serial_id: i64,
    pub quantity: BigDecimal,
    pub total_cost: BigDecimal,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CostApplication {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub application_type: String,   // "DirectLabor" | "Overhead"
    pub amount: BigDecimal,
    pub applied_at: DateTime<Utc>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VarianceProrationResult {
    pub gl_transaction_uuid: Uuid,
    pub wip_alloc: BigDecimal,
    pub fg_alloc: BigDecimal,
    pub cogs_alloc: BigDecimal,
    pub total_allocated: BigDecimal,
}

// DTOs for input
#[derive(Debug, Deserialize)]
pub struct CreateProductionOrderDto {
    pub order_number: String,
    pub product_item_serial_id: i64,
    pub quantity_ordered: BigDecimal,
    pub start_date: Option<NaiveDate>,
    pub expected_completion_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct IssueMaterialDto {
    pub production_order_uuid: Uuid,
    pub item_serial_id: i64,
    pub warehouse_serial_id: i64,
    pub quantity: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct ApplyOverheadDto {
    pub production_order_uuid: Uuid,
    pub base_amount: BigDecimal,   // e.g. labor hours or labor cost
}

#[derive(Debug, Deserialize)]
pub struct CompleteProductionOrderDto {
    pub production_order_uuid: Uuid,
    pub completed_quantity: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct ProrateVarianceDto {
    pub variance_amount: BigDecimal,           // positive = under-applied
    pub as_of_date: Option<NaiveDate>,
    pub memo: Option<String>,
    pub dry_run: Option<bool>,
}