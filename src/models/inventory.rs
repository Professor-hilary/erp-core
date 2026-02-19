// src/models/item.rs
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Item {
    pub uuid: Uuid,                   // Default uuid
    pub serial_id: Option<i64>,       // Integer ID for easy query
    pub warehouse_serial: i64,        // Required to track movements
    pub sku: String,                  // Useful for user
    pub name: String,                 // Required
    pub category_uuid: Option<Uuid>,  // Based on whatever criteria
    pub description: Option<String>,  // Relevant but not necessary
    pub unit: String,                 // UOM e.g. boxes, litres, pieces...
    pub selling_price: BigDecimal,    // Selling price, will soon remove this
    pub track_quantity: bool,         // Relevant for the future
    pub reorder_level: BigDecimal,    // When to make new purchase order
    pub asset_account: String,        // Account for tracking asset
    pub cogs_account: Option<String>, // Account for expensing purchases
    pub income_account: String,       // Sales or revenue account
    pub status: String,               // ...?
    pub created_at: DateTime<Utc>,    // Useful for timeseries
    pub updated_at: DateTime<Utc>,    // Useful for timeseries
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct ItemCategory {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Warehouse {
    pub uuid: Uuid,
    pub serial_id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub location: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemCategory {
    pub code: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub sku: String,
    pub name: String,
    pub warehouse_serial: i64,
    pub category_uuid: Option<Uuid>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub selling_price: Option<BigDecimal>,
    pub track_quantity: Option<bool>,
    pub reorder_level: Option<BigDecimal>,
    pub asset_account: Option<String>,
    pub cogs_account: Option<String>,
    pub income_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWarehouse {
    pub code: String,
    pub name: String,
    pub description: String,
    pub location: String,
}

// Manufacturing section: Production Order, BOM,...
#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct ProductionOrder {
    pub uuid: Uuid,
    pub order_number: String,
    pub product_item_id: i64,
    pub quantity_ordered: BigDecimal,
    pub quantity_completed: BigDecimal,
    pub status: String,
    pub expected_completion_date: Option<NaiveDate>,
    pub actual_completion_date: Option<NaiveDate>,
}

#[derive(FromRow, Debug, Deserialize, Serialize, Clone)]
pub struct CreateProductionOrder {
    pub uuid: Uuid,
    pub order_number: String,
    pub product_item_id: i64,
    pub quantity_ordered: BigDecimal,
    pub quantity_completed: BigDecimal,
    pub status: String,
    pub expected_completion_date: Option<NaiveDate>,
    pub actual_completion_date: Option<NaiveDate>,
}

#[derive(FromRow, Debug, Deserialize, Serialize, Clone)]
pub struct BillOfmaterial {
    pub uuid: String,
    pub bom_header_uuid: String,
    pub component_item_uuid: String,
    pub quantity_per: String,
    pub unit_of_measure: String,
    pub scrap_factor: String,
    pub line_number: String,
}
