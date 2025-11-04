use serde::Deserialize;

// src/models/inventory_movement.rs
#[derive(Debug, Deserialize)]
pub struct PostPurchase {
    pub item_id: i64,
    pub quantity: bigdecimal::BigDecimal,
    pub unit_cost: bigdecimal::BigDecimal,
    pub reference_type: String,
    pub reference_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct PostSale {
    pub item_id: i64,
    pub quantity: bigdecimal::BigDecimal,
    pub unit_cost: bigdecimal::BigDecimal,
    pub reference_type: String,
    pub reference_id: i64,
}