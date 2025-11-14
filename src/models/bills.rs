use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateBill {
    pub bill_number: String,
    pub vendor_id: i64,
    pub issue_date: NaiveDate,
    pub due_date: NaiveDate,
    pub total: bigdecimal::BigDecimal,
    pub items: Option<serde_json::Value>, // JSONB array
}
