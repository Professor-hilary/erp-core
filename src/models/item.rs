// src/models/item.rs (for inventory)
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct Item {
    pub uuid: Uuid,
    pub sku: String,
    pub name: String,
    pub category_uuid: Option<Uuid>,
    pub description: Option<String>,
    pub unit: String,
    pub cost_price: bigdecimal::BigDecimal,
    pub selling_price: bigdecimal::BigDecimal,
    pub track_quantity: bool,
    pub quantity_on_hand: bigdecimal::BigDecimal,
    pub reorder_level: bigdecimal::BigDecimal,
    pub asset_account: String,
    pub cogs_account: String,
    pub income_account: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub sku: String,
    pub name: String,
    pub category_uuid: Option<Uuid>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub cost_price: Option<bigdecimal::BigDecimal>,
    pub selling_price: Option<bigdecimal::BigDecimal>,
    pub track_quantity: Option<bool>,
    pub reorder_level: Option<bigdecimal::BigDecimal>,
    pub asset_account: Option<String>,
    pub cogs_account: Option<String>,
    pub income_account: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostPurchase {
    pub item_id: Uuid,
    pub quantity: bigdecimal::BigDecimal,
    pub unit_cost: bigdecimal::BigDecimal,
    pub reference_type: String,
    pub reference_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct PostSale {
    pub item_id: Uuid,
    pub quantity: bigdecimal::BigDecimal,
    pub unit_cost: bigdecimal::BigDecimal,
    pub reference_type: String,
    pub reference_id: Uuid,
}
