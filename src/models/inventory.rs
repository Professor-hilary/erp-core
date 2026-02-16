// src/models/item.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Item {
    pub uuid: Uuid,                            // Default uuid
    pub serial_id: Option<i64>,                // Integer ID for easy query
    pub warehouse_serial: i64,                 // Required to track movements
    pub sku: String,                           // Useful for user
    pub name: String,                          // Required
    pub category_uuid: Option<Uuid>,           // Based on whatever criteria
    pub description: Option<String>,           // Relevant but not necessary
    pub unit: String,                          // UOM e.g. boxes, litres, pieces...
    pub selling_price: bigdecimal::BigDecimal, // Selling price, will soon remove this
    pub track_quantity: bool,                  // Relevant for the future
    pub reorder_level: bigdecimal::BigDecimal, // When to make new purchase order
    pub asset_account: String,                 // Account for tracking asset
    pub cogs_account: Option<String>,          // Account for expensing purchases
    pub income_account: String,                // Sales or revenue account
    pub status: String,                        // ...?
    pub created_at: DateTime<Utc>,             // Useful for timeseries
    pub updated_at: DateTime<Utc>,             // Useful for timeseries
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
    pub selling_price: Option<bigdecimal::BigDecimal>,
    pub track_quantity: Option<bool>,
    pub reorder_level: Option<bigdecimal::BigDecimal>,
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

// #[derive(Debug, Deserialize)]
// pub struct PostSale {
//     pub item_serial_id: i64,
//     pub warehouse_serial_id: Option<i64>,
//     pub quantity: bigdecimal::BigDecimal,
//     pub unit_cost: bigdecimal::BigDecimal,
//     pub reference_type: Option<String>,
//     pub reference_serial_id: Option<i64>,
//     pub cash_account_code: String,
// }
