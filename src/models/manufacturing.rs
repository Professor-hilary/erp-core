// src/models/manufacturing.rs

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub order_number: String,
    pub product_item_serial_id: i64, // links to inventory.items.serial_id
    pub quantity_ordered: BigDecimal,
    pub quantity_completed: BigDecimal,
    pub status: String, // "Planned" | "In Progress" | "Completed" | "Cancelled"
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
    pub application_type: String, // "DirectLabor" | "Overhead"
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
    pub base_amount: BigDecimal, // e.g. labor hours or labor cost
    pub wip_account: String,
    pub overhead_control_account: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyLaborCostDto {
    pub production_order_uuid: Uuid, // Current inventory in production
    pub hours: BigDecimal,           // e.g. labor hours or labor cost
    pub rate_per_hour: BigDecimal,   // e.g. labor hours or labor cost
    pub is_direct: bool,             // Direct or Indirect labor
    pub wip_account_code: String,    // Work In Progress to accumulate costs
    pub overhead_applied: String,    // Overhead account
    pub overhead_control: String,    // Production Expenses tracker
    pub reference: Option<String>,   // Message
    pub department_code: Option<String>, // Optional department
}

#[derive(Debug, Deserialize)]
pub struct CompleteProductionOrderDto {
    pub production_order_uuid: Uuid,
    pub completed_quantity: BigDecimal,
    pub wip_account_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ProrateVarianceDto {
    pub variance_amount: BigDecimal, // positive = under-applied
    pub as_of_date: Option<NaiveDate>,
    pub wip_account: Option<String>,
    pub fg_account: Option<String>,
    pub cogs_account: Option<String>,
    pub memo: Option<String>,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BomHeader {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub bom_code: String,
    pub product_item_uuid: Uuid,
    pub description: Option<String>,
    pub revision: String,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

// Overhead Rates DTOs
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OverheadRates {
    pub uuid: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub allocation_base: String,
    pub estimated_overhead: BigDecimal,
    pub estimated_base: BigDecimal,
    pub rate: BigDecimal,
    pub department_code: String,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOverheadRateDto {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub allocation_base: String,
    pub estimated_overhead: BigDecimal,
    pub estimated_base: BigDecimal,
    pub department_code: Option<String>,
    pub is_active: Option<bool>,
}

// Bill Of Material DTOs
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BomLine {
    pub uuid: Uuid,
    pub bom_header_uuid: Uuid,
    pub line_number: i16,
    pub component_item_uuid: Uuid,
    pub quantity_per: BigDecimal,
    pub uom: String,
    pub scrap_factor: BigDecimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomHeaderDto {
    pub bom_code: String,
    pub product_item_id: i64,
    pub description: Option<String>,
    pub revision: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBomLineDto {
    pub line_number: i16,
    pub component_item_uuid: Uuid,
    pub quantity_per: BigDecimal,
    pub uom: Option<String>,
    pub scrap_factor: Option<BigDecimal>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BomWithLines {
    pub header: BomHeader,
    pub lines: Vec<BomLine>,
}
