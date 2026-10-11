mod common;

use common::{E2eResult, TestContext};
use common::client::extract_uuid;
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn procure_to_pay() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    // Ensure we have cash / equity / AP context from earlier phases
    // (safe if already posted — just balances the suite story)
    if ctx.vendors.is_empty() {
        ctx.seed_trading_partners()
            .await
            .map_err(|e| e.to_string())?;
    }
    assert!(
        !ctx.vendors.is_empty(),
        "need at least one vendor — run e2e_02 first"
    );

    let vendor_uuid = *ctx.vendors.values().next().expect("vendor");

    // ----- 1. Warehouse -----
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/warehouse",
            json!({
                "code": "WH-MAIN",
                "name": "Main Warehouse",
                "description": "Primary store",
                "location": "Kampala"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create warehouse: {s} {body}");
    let warehouse_serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("warehouse serial_id missing")?;
    println!("✓ warehouse serial={warehouse_serial}");

    // ----- 2. Category -----
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/category",
            json!({
                "code": "RAW",
                "name": "Raw Materials",
                "description": "Inputs for production"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create category: {s} {body}");
    let category_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ category {category_uuid}");

    // ----- 3. Stock item -----
    // asset_account / cogs / income = COA codes (strings)
    let inv_code = if ctx.account_uuid("110701").is_some() {
        "110701"
    } else {
        "110700"
    };
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/item",
            json!({
                "sku": "RM-FLOUR-25KG",
                "name": "Wheat Flour 25kg",
                "warehouse_serial": warehouse_serial,
                "category_uuid": category_uuid,
                "description": "Bakery flour",
                "unit": "bag",
                "selling_price": "45000",
                "standard_cost": "32000",
                "track_quantity": true,
                "reorder_level": "10",
                "asset_account": inv_code,
                "cogs_account": "510100",
                "income_account": "410100"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create item: {s} {body}");
    let stock_item_id = body["data"]["serial_id"]
        .as_i64()
        .ok_or("item serial_id missing")?;
    println!("✓ item serial={stock_item_id}");

    // ----- 4. Create purchase bill (unposted) -----
    let qty = "50";
    let unit_price = "32000";
    let tax_rate = "18"; // %
    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/create/purchase",
            json!({
                "bill_number": format!("BILL-{}", chrono::Utc::now().timestamp_millis()),
                "vendor_uuid": vendor_uuid,
                "bill_date": "2025-01-20",
                "due_date": "2025-02-20",
                "reference": "PO-E2E-001",
                "settlement_type": "credit",
                "payment_status": "unpaid",
                "items": [{
                    "stock_item_id": stock_item_id,
                    "description": "Wheat Flour 25kg",
                    "quantity": qty,
                    "unit_price": unit_price,
                    "tax_rate": tax_rate
                }]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create purchase: {s} {body}");
    let bill_serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("bill serial_id missing")?;
    let bill_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ bill serial={bill_serial} uuid={bill_uuid}");

    // ----- 5. Post credit purchase → inventory + input VAT + AP -----
    // Adjust VAT code if your COA uses a different input-VAT account.
    let vat_input = "110800"; // common placeholder; change if your COA differs
    let ap_code = "210100";

    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/post/credit/purchase",
            json!({
                "bill_serial_id": bill_serial,
                "vat_tax_account": vat_input,
                "payables_account": ap_code,
                "cash_account": null
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post credit purchase: {s} {body}");
    println!("✓ credit purchase posted");

    // ----- 6. Vendor bills list -----
    let (s, body) = ctx
        .client
        .get(&format!("/api/procurement/list/vendor-bills/{vendor_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list vendor bills: {s} {body}");
    let bills = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(bills >= 1, "expected at least one vendor bill");
    println!("✓ vendor bills n={bills}");

    // ----- 7. Trial balance still healthy -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after procure-to-pay");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ procure_to_pay OK for {}", ctx.company_name);
    Ok(())
}

