// src/models/manufacturing.rs

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// use strum::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProductionOrder {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub order_number: String,
    pub product_item_uuid: Uuid, // links to inventory.items.serial_id
    pub quantity_ordered: BigDecimal,
    pub quantity_completed: BigDecimal,
    pub status: String, // "Planned" | "In Progress" | "Completed" | "Cancelled"
    pub start_date: Option<NaiveDate>,
    pub expected_completion_date: Option<NaiveDate>,
    pub actual_completion_date: Option<NaiveDate>,
    pub bom_header_uuid: Option<Uuid>,
    pub routing_uuid: Option<Uuid>,
    pub standard_unit_cost: Option<BigDecimal>,
    pub fg_warehouse_serial: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkCenter {
    pub uuid: Uuid,
    pub name: String,
    pub code: String,
    pub labor_rate: BigDecimal,
    pub allocation_base: String,
    pub department_code: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct WorkCenterDto {
    pub code: String,
    pub name: String,
    pub labor_rate: BigDecimal,
    pub allocation_base: String,
    pub department_code: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Routings {
    pub uuid: Uuid,
    pub product_item_uuid: Uuid,
    pub routing_code: String,
    pub notes: String,
    pub version: String,
    pub base_quantity: BigDecimal,
    pub effective_date: NaiveDate,
    pub status: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RoutingsDto {
    pub product_item_uuid: Uuid,
    pub routing_code: String,
    pub notes: Option<String>,
    pub version: String,
    pub base_quantity: BigDecimal,
    pub effective_date: NaiveDate,
    pub status: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RoutingOperations {
    pub uuid: Uuid,
    pub routing_uuid: Uuid,
    pub sequence: i16,
    pub operation_name: String,
    pub work_center: String,
    pub description: String,
    pub setup_time_minutes: BigDecimal,
    pub run_time_minutes: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RoutingOperationsDto {
    pub routing_uuid: Uuid,
    pub sequence: i16,
    pub operation_name: String,
    pub work_center: String,
    pub description: Option<String>,
    pub setup_time_minutes: BigDecimal,
    pub run_time_minutes: BigDecimal,
}

// #[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
// #[strum(serialize_all = "lowercase")]
// pub enum MaterialStatus {
//     Pending,
//     PartiallyIssued,
//     FullyIssued,
// }

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BomMaterialIssue {
    pub component_item_uuid: Uuid,
    pub required_qty: BigDecimal,
    pub issued_qty: BigDecimal,
    pub remaining_qty: BigDecimal,
    pub material_status: String, // MaterialStatus,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FullMaterialIssue {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub stock_item_id: i64,
    pub stock_item_name: String,
    pub quantity: BigDecimal,
    pub unit_cost: BigDecimal,
    pub total_cost: BigDecimal,
    pub warehouse_uuid: Uuid,
    pub issued_at: DateTime<Utc>, // MaterialStatus,
}

#[derive(Debug, Deserialize)]
pub struct SingleMaterialIssueDto {
    pub item_serial_id: i64,
    pub warehouse_serial: i64,
    pub quantity: BigDecimal,
    pub production_order_uuid: Uuid,
    // pub raw_materials_code: String,
    // pub work_in_progress_code: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CostApplication {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "application_type")] // keep API name if you want
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
    pub product_item_uuid: Uuid,
    pub quantity_ordered: BigDecimal,
    pub start_date: Option<NaiveDate>,
    pub expected_completion_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct IssueMaterialDto {
    pub warehouse_serial_id: i64,
    pub production_order_uuid: Uuid,
    /// Optional — resolved from work_in_progress role when omitted
    #[serde(default)]
    pub wip_account_code: Option<String>,
    /// Optional — resolved from raw_materials_inventory role when omitted
    #[serde(default)]
    pub raw_mat_account_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyOverheadDto {
    pub production_order: String,
    pub base_hours: BigDecimal,
    #[serde(default)]
    pub wip_account: Option<String>,
    #[serde(default)]
    pub overhead_control_account: Option<String>,
    pub allocation_base: String,
}

#[derive(Debug, Deserialize)]
pub struct RecognizeOverhead {
    pub production_order_uuid: Uuid,
    pub base_amount: BigDecimal,
    #[serde(default)]
    pub payable_or_cash: Option<String>,
    #[serde(default)]
    pub overhead_control_account: Option<String>,
    pub allocation_base: String,
    pub reference: String,
}

#[derive(Debug, Deserialize)]
pub struct ApplyDirectLaborDto {
    pub production_order_uuid: Uuid,
    /// Optional — wages_payable / accrued wages
    #[serde(default)]
    pub labor_account_code: Option<String>,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    pub reference: Option<String>,
}

// #[derive(Debug, Deserialize)]
// pub struct ApplyInDirectLaborDto {
//     pub hours: BigDecimal,          // e.g. labor hours or labor cost
//     pub rate_per_hour: BigDecimal,  // e.g. labor hours or labor cost
//     pub labor_account_code: String, // Salaries or wages payable code
//     // pub wip_account_code: Option<String>, // Work In Progress to accumulate costs
//     pub control_account_code: Option<String>, // Production Expenses tracker - strictly overheads
//     pub reference: Option<String>,            // Memo
//     pub department_code: Option<String>,      // Optional department
// }

#[derive(Debug, Deserialize)]
pub struct StartProductionOrderDto {
    pub production_order_uuid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CompleteProductionOrderDto {
    pub production_order_uuid: Uuid,
    pub completed_quantity: Option<BigDecimal>,
    /// Optional FG destination warehouse serial; falls back to order pin then product default
    #[serde(default)]
    pub fg_warehouse_serial: Option<i64>,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub fg_account_code: Option<String>,
    #[serde(default)]
    pub mfg_mat_var_code: Option<String>,
    #[serde(default)]
    pub mfg_lab_var_code: Option<String>,
    #[serde(default)]
    pub mfg_moh_var_code: Option<String>,
    #[serde(default)]
    pub control_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProrateVarianceDto {
    pub variance_amount: BigDecimal, // positive = under-applied
    pub as_of_date: Option<NaiveDate>,
    pub wip_account: Option<String>,
    pub fg_account: Option<String>,
    pub cogs_account: Option<String>,
    pub min_allocation_threshold: Option<BigDecimal>,
    pub materiality_threshold: Option<BigDecimal>,
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
    pub department_code: Option<String>,
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
// #[derive(Debug, Clone, FromRow, Serialize)]
// pub struct BomLine {
//     pub uuid: Uuid,
//     pub bom_header_uuid: Uuid,
//     pub line_number: i16,
//     pub component_item_uuid: Uuid,
//     pub quantity_per: BigDecimal,
//     pub uom: String,
//     pub scrap_factor: BigDecimal,
//     pub notes: Option<String>,
//     pub created_at: DateTime<Utc>,
// }

#[derive(Debug, Deserialize)]
pub struct CreateBomHeaderDto {
    pub bom_code: String,
    pub product_item_uuid: Uuid,
    pub description: Option<String>,
    pub revision: Option<String>,
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
}

// #[derive(Debug, Deserialize)]
// pub struct CreateBomLineDto {
//     pub line_number: i16,
//     pub component_item_uuid: Uuid,
//     pub quantity_per: BigDecimal,
//     pub uom: Option<String>,
//     pub scrap_factor: Option<BigDecimal>,
//     pub notes: Option<String>,
// }

#[derive(Debug, Serialize)]
pub struct BomWithLines {
    pub header: BomHeader,
    pub lines: Vec<BomLine>,
}

// --- BomLine (add fields) ---
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BomLine {
    pub uuid: Uuid,
    pub bom_header_uuid: Option<Uuid>,
    pub line_number: i16,
    pub component_item_uuid: Uuid,
    pub quantity_per: BigDecimal,
    pub uom: String,
    pub scrap_factor: BigDecimal,
    pub notes: Option<String>,
    pub child_bom_uuid: Option<Uuid>, // NEW
    pub is_phantom: bool,             // NEW
    pub created_at: DateTime<Utc>,
}

// --- CreateBomLineDto ---
#[derive(Debug, Deserialize)]
pub struct CreateBomLineDto {
    pub line_number: i16,
    pub component_item_uuid: Uuid,
    pub quantity_per: BigDecimal,
    pub uom: Option<String>,
    pub scrap_factor: Option<BigDecimal>,
    pub notes: Option<String>,
    pub child_bom_uuid: Option<Uuid>, // NEW – pin a specific sub-BOM revision
    pub is_phantom: Option<bool>,     // NEW
}

// Optional: useful for an explode endpoint later
// #[derive(Debug, Clone, FromRow, Serialize)]
// pub struct ExplodedBomLine {
//     pub level: i32,
//     pub component_item_uuid: Uuid,
//     pub quantity_required: BigDecimal,
//     pub is_leaf: bool,
//     pub is_phantom: bool,
//     pub source_bom_uuid: Uuid,
// }

/// Draft or posted actual labour allocation (time / payroll), distinct from routing-applied labour.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourAllocation {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub labour_class: String, // Direct | Indirect
    pub hours: BigDecimal,
    pub rate: BigDecimal,
    pub amount: BigDecimal,
    pub work_center_code: Option<String>,
    pub employee_uuid: Option<Uuid>,
    pub source: String, // Manual | Payroll | TimeEntry
    pub source_ref: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub cost_application_uuid: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabourAllocationDto {
    pub production_order_uuid: Uuid,
    /// Direct | Indirect
    pub labour_class: String,
    pub hours: BigDecimal,
    pub rate: BigDecimal,
    #[serde(default)]
    pub work_center_code: Option<String>,
    #[serde(default)]
    pub employee_uuid: Option<Uuid>,
    /// Manual | Payroll | TimeEntry
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostLabourAllocationDto {
    pub allocation_uuid: Uuid,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub wages_account_code: Option<String>,
    /// Required for Indirect when charging OH control instead of WIP
    #[serde(default)]
    pub overhead_control_account: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OperationExecution {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub routing_operation_uuid: Option<Uuid>,
    pub sequence: Option<i16>,
    pub operation_name: String,
    pub work_center_code: Option<String>,
    pub status: String,
    pub qty_started: Option<BigDecimal>,
    pub qty_completed: Option<BigDecimal>,
    pub qty_scrapped: Option<BigDecimal>,
    pub qty_reworked: Option<BigDecimal>,
    pub setup_minutes_actual: Option<BigDecimal>,
    pub run_minutes_actual: Option<BigDecimal>,
    pub machine_minutes_actual: Option<BigDecimal>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub operator_uuid: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EnsureOperationsDto {
    pub production_order_uuid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StartOperationDto {
    pub execution_uuid: Uuid,
    #[serde(default)]
    pub qty_started: Option<BigDecimal>,
    #[serde(default)]
    pub operator_uuid: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteOperationDto {
    pub execution_uuid: Uuid,
    pub qty_completed: BigDecimal,
    #[serde(default)]
    pub qty_scrapped: Option<BigDecimal>,
    #[serde(default)]
    pub setup_minutes_actual: Option<BigDecimal>,
    #[serde(default)]
    pub run_minutes_actual: Option<BigDecimal>,
    #[serde(default)]
    pub machine_minutes_actual: Option<BigDecimal>,
    #[serde(default)]
    pub operator_uuid: Option<Uuid>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ScrapReworkEvent {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub operation_execution_uuid: Option<Uuid>,
    pub event_type: String,
    pub quantity: BigDecimal,
    pub reason_code: Option<String>,
    pub reason_text: Option<String>,
    pub disposition: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RecordScrapReworkDto {
    pub production_order_uuid: Uuid,
    pub event_type: String,
    pub quantity: BigDecimal,
    #[serde(default)]
    pub operation_execution_uuid: Option<Uuid>,
    #[serde(default)]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub reason_text: Option<String>,
    #[serde(default)]
    pub disposition: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct QualityInspection {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub operation_execution_uuid: Option<Uuid>,
    pub inspection_type: String,
    pub qty_inspected: BigDecimal,
    pub qty_passed: BigDecimal,
    pub qty_failed: BigDecimal,
    pub result: String,
    pub inspector_uuid: Option<Uuid>,
    pub notes: Option<String>,
    pub inspected_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RecordQualityInspectionDto {
    pub production_order_uuid: Uuid,
    pub qty_inspected: BigDecimal,
    pub qty_passed: BigDecimal,
    pub qty_failed: BigDecimal,
    pub result: String,
    #[serde(default)]
    pub inspection_type: Option<String>,
    #[serde(default)]
    pub operation_execution_uuid: Option<Uuid>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourCategory {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabourCategoryDto {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourRate {
    pub uuid: Uuid,
    pub labour_category_uuid: Option<Uuid>,
    pub employee_uuid: Option<Uuid>,
    pub work_center_code: Option<String>,
    pub rate_kind: String, // Standard | Actual
    pub hourly_rate: BigDecimal,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabourRateDto {
    #[serde(default)]
    pub labour_category_uuid: Option<Uuid>,
    #[serde(default)]
    pub employee_uuid: Option<Uuid>,
    #[serde(default)]
    pub work_center_code: Option<String>,
    /// Standard | Actual
    pub rate_kind: String,
    pub hourly_rate: BigDecimal,
    pub effective_from: chrono::NaiveDate,
    #[serde(default)]
    pub effective_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourTimeEntry {
    pub uuid: Uuid,
    pub employee_uuid: Uuid,
    pub production_order_uuid: Option<Uuid>,
    pub operation_execution_uuid: Option<Uuid>,
    pub work_center_code: Option<String>,
    pub labour_category_uuid: Option<Uuid>,
    pub work_date: chrono::NaiveDate,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub break_minutes: BigDecimal,
    pub hours: Option<BigDecimal>,
    pub activity_type: String,
    pub quantity_completed: Option<BigDecimal>,
    pub quantity_scrapped: Option<BigDecimal>,
    pub status: String,
    pub hourly_rate: Option<BigDecimal>,
    pub amount: Option<BigDecimal>,
    pub memo: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub rejection_reason: Option<String>,
    pub labour_allocation_uuid: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabourTimeEntryDto {
    pub employee_uuid: Uuid,
    pub work_date: chrono::NaiveDate,
    #[serde(default)]
    pub hours: Option<BigDecimal>,
    #[serde(default)]
    pub activity_type: Option<String>,
    #[serde(default)]
    pub production_order_uuid: Option<Uuid>,
    #[serde(default)]
    pub operation_execution_uuid: Option<Uuid>,
    #[serde(default)]
    pub work_center_code: Option<String>,
    #[serde(default)]
    pub labour_category_uuid: Option<Uuid>,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub break_minutes: Option<BigDecimal>,
    #[serde(default)]
    pub quantity_completed: Option<BigDecimal>,
    #[serde(default)]
    pub quantity_scrapped: Option<BigDecimal>,
    #[serde(default)]
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApproveLabourTimeEntryDto {
    pub entry_uuid: Uuid,
    #[serde(default)]
    pub hourly_rate: Option<BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct RejectLabourTimeEntryDto {
    pub entry_uuid: Uuid,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostLabourTimeEntryDto {
    pub entry_uuid: Uuid,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub wages_account_code: Option<String>,
    #[serde(default)]
    pub overhead_control_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DraftTimeFromOperationDto {
    pub execution_uuid: Uuid,
    #[serde(default)]
    pub employee_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProductionOrderLabourSummary {
    pub production_order_uuid: Uuid,
    pub order_number: String,
    pub order_status: Option<String>,
    pub applied_labour: BigDecimal,
    pub actual_labour: BigDecimal,
    pub indirect_labour: BigDecimal,
    pub actual_direct_hours: BigDecimal,
    pub open_timesheet_hours: BigDecimal,
    pub labour_cost_variance: BigDecimal,
    pub standard_hours: Option<BigDecimal>,
    pub variance_actual_hours: Option<BigDecimal>,
    pub standard_rate: Option<BigDecimal>,
    pub actual_rate: Option<BigDecimal>,
    pub rate_variance: Option<BigDecimal>,
    pub efficiency_variance: Option<BigDecimal>,
    pub last_computed_total_variance: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourVariance {
    pub uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub standard_hours: BigDecimal,
    pub actual_hours: BigDecimal,
    pub standard_rate: BigDecimal,
    pub actual_rate: BigDecimal,
    pub applied_labour_cost: BigDecimal,
    pub actual_labour_cost: BigDecimal,
    pub rate_variance: BigDecimal,
    pub efficiency_variance: BigDecimal,
    pub total_variance: BigDecimal,
    pub posted: bool,
    pub cost_application_uuid: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourProductivityByOrder {
    pub production_order_uuid: Uuid,
    pub order_number: String,
    pub status: Option<String>,
    pub quantity_ordered: Option<BigDecimal>,
    pub quantity_completed: Option<BigDecimal>,
    pub direct_hours: BigDecimal,
    pub non_productive_hours: BigDecimal,
    pub timesheet_cost: BigDecimal,
    pub units_per_direct_hour: Option<BigDecimal>,
    pub actual_labour_cost_per_unit: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourByEmployee {
    pub employee_uuid: Uuid,
    pub month_start: chrono::NaiveDate,
    pub posted_hours: BigDecimal,
    pub productive_hours: BigDecimal,
    pub non_productive_hours: BigDecimal,
    pub posted_cost: BigDecimal,
    pub utilisation_pct: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourByWorkCenter {
    pub work_center_code: String,
    pub month_start: chrono::NaiveDate,
    pub posted_entries: i64,
    pub posted_hours: BigDecimal,
    pub posted_cost: BigDecimal,
    pub qty_completed: BigDecimal,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OperationEfficiency {
    pub execution_uuid: Uuid,
    pub production_order_uuid: Uuid,
    pub order_number: Option<String>,
    pub sequence: Option<i16>,
    pub operation_name: Option<String>,
    pub work_center_code: Option<String>,
    pub status: Option<String>,
    pub actual_minutes: BigDecimal,
    pub standard_minutes: Option<BigDecimal>,
    pub efficiency_pct: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AttendanceEntry {
    pub uuid: Uuid,
    pub employee_uuid: Uuid,
    pub work_date: chrono::NaiveDate,
    pub status: String,
    pub clock_in: Option<DateTime<Utc>>,
    pub clock_out: Option<DateTime<Utc>>,
    pub break_minutes: BigDecimal,
    pub hours_present: Option<BigDecimal>,
    pub source: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAttendanceDto {
    pub employee_uuid: Uuid,
    pub work_date: chrono::NaiveDate,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub clock_in: Option<DateTime<Utc>>,
    #[serde(default)]
    pub clock_out: Option<DateTime<Utc>>,
    #[serde(default)]
    pub break_minutes: Option<BigDecimal>,
    #[serde(default)]
    pub hours_present: Option<BigDecimal>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourHoursReconciliation {
    pub employee_uuid: Uuid,
    pub work_date: chrono::NaiveDate,
    pub attendance_hours: BigDecimal,
    pub booked_hours: BigDecimal,
    pub productive_hours: BigDecimal,
    pub unallocated_hours: BigDecimal,
    pub booking_coverage_pct: Option<BigDecimal>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LabourHoursReconciliationPeriod {
    pub employee_uuid: Uuid,
    pub month_start: chrono::NaiveDate,
    pub attendance_hours: BigDecimal,
    pub booked_hours: BigDecimal,
    pub productive_hours: BigDecimal,
    pub unallocated_hours: BigDecimal,
    pub booking_coverage_pct: Option<BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct AllocatePayslipLabourDto {
    pub payslip_uuid: Uuid,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub labour_clearing_account: Option<String>,
    /// employer_cost | gross | basic
    #[serde(default)]
    pub cost_basis: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayslipLabourAllocationRow {
    pub production_order_uuid: Uuid,
    pub allocated_amount: BigDecimal,
    pub hours_share: BigDecimal,
    pub cost_application_uuid: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeamTimeEntriesDto {
    pub employee_uuids: Vec<Uuid>,
    pub work_date: chrono::NaiveDate,
    pub hours: BigDecimal,
    #[serde(default)]
    pub activity_type: Option<String>,
    #[serde(default)]
    pub production_order_uuid: Option<Uuid>,
    #[serde(default)]
    pub operation_execution_uuid: Option<Uuid>,
    #[serde(default)]
    pub work_center_code: Option<String>,
    #[serde(default)]
    pub labour_category_uuid: Option<Uuid>,
    #[serde(default)]
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkTimeEntryIdsDto {
    pub entry_uuids: Vec<Uuid>,
    #[serde(default)]
    pub hourly_rate: Option<BigDecimal>,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub wages_account_code: Option<String>,
    #[serde(default)]
    pub overhead_control_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AllocatePayrunLabourDto {
    pub payrun_uuid: Uuid,
    #[serde(default)]
    pub wip_account_code: Option<String>,
    #[serde(default)]
    pub labour_clearing_account: Option<String>,
    #[serde(default)]
    pub cost_basis: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PayrunLabourAllocationRow {
    pub payslip_uuid: Uuid,
    pub production_order_uuid: Option<Uuid>,
    pub allocated_amount: BigDecimal,
    pub hours_share: BigDecimal,
    pub cost_application_uuid: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DraftTimeFromOrderDto {
    pub production_order_uuid: Uuid,
}


/// Tenant manufacturing costing method (IAS / cost-accounting policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostingMethod {
    Actual,
    Normal,
    Standard,
}

impl CostingMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Actual => "actual",
            Self::Normal => "normal",
            Self::Standard => "standard",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(Self::Actual),
            "normal" => Some(Self::Normal),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OhApplicationBase {
    LaborHours,
    MachineHours,
    Units,
}

impl OhApplicationBase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LaborHours => "labor_hours",
            Self::MachineHours => "machine_hours",
            Self::Units => "units",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "labor_hours" | "labour_hours" => Some(Self::LaborHours),
            "machine_hours" => Some(Self::MachineHours),
            "units" => Some(Self::Units),
            _ => None,
        }
    }
}

/// Loaded costing policy for the tenant (singleton row in Stage A).
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CostingPolicy {
    pub uuid: Uuid,
    pub costing_method: String,
    pub oh_application_base: String,
    pub use_timesheet_actual: bool,
    pub use_payroll_allocation: bool,
    pub effective_from: NaiveDate,
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

impl CostingPolicy {
    pub fn method(&self) -> CostingMethod {
        CostingMethod::parse(&self.costing_method).unwrap_or(CostingMethod::Standard)
    }

    pub fn oh_base(&self) -> OhApplicationBase {
        OhApplicationBase::parse(&self.oh_application_base).unwrap_or(OhApplicationBase::LaborHours)
    }

    /// Routing / standard-hour applied labour posts product cost only under standard costing.
    pub fn allows_routing_applied_labor(&self) -> bool {
        matches!(self.method(), CostingMethod::Standard)
    }

    /// Predetermined OH rate application (Dr WIP / Cr OH applied).
    /// Actual costing records OH only via recognize/actual paths.
    pub fn allows_predetermined_overhead(&self) -> bool {
        matches!(
            self.method(),
            CostingMethod::Standard | CostingMethod::Normal
        )
    }

    /// Material/labour/OH standard-cost variance posts on completion & variance APIs.
    pub fn posts_standard_variances(&self) -> bool {
        matches!(self.method(), CostingMethod::Standard)
    }

    /// Shop-floor timesheets may post ActualLabor when enabled on the policy.
    pub fn allows_timesheet_actual(&self) -> bool {
        self.use_timesheet_actual
    }

    /// Payroll → production allocation when enabled on the policy.
    pub fn allows_payroll_allocation(&self) -> bool {
        self.use_payroll_allocation
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCostingPolicyDto {
    pub costing_method: Option<String>,
    pub oh_application_base: Option<String>,
    pub use_timesheet_actual: Option<bool>,
    pub use_payroll_allocation: Option<bool>,
    pub effective_from: Option<NaiveDate>,
    pub notes: Option<String>,
}
