mod common;

use common::client::extract_uuid;
use common::{E2eResult, TestContext};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

fn acct(ctx: &TestContext, codes: &[&str]) -> Result<String, String> {
    for c in codes {
        if ctx.account_uuid(c).is_some() {
            return Ok(c.to_string());
        }
    }
    // Still return the first preferred code — post may create clearer errors
    Ok(codes[0].to_string())
}

#[tokio::test]
async fn manufacturing_flow() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // ----- Inventory baseline: warehouse + raw material -----
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inventory: {s} {inv}");

    let (rm_uuid, rm_serial, warehouse_serial) =
        if let Some(item) = inv["data"].as_array().and_then(|a| a.first()) {
            let uuid = extract_uuid(item, &["uuid"]).map_err(|e| e.to_string())?;
            let serial = item["serial_id"].as_i64().ok_or("rm serial_id")?;
            let wh = item["warehouse_serial"].as_i64().unwrap_or(1);
            println!("✓ reuse RM serial={serial} uuid={uuid}");
            (uuid, serial, wh)
        } else {
            // Bootstrap warehouse + RM if suite was run out of order
            let (s, body) = ctx
                .client
                .post(
                    "/api/inventory/create/warehouse",
                    json!({
                        "code": "WH-MFG",
                        "name": "Production Store",
                        "description": "Manufacturing",
                        "location": "Kampala"
                    }),
                )
                .await
                .map_err(|e| e.to_string())?;
            assert!(ok(s), "warehouse: {s} {body}");
            let wh = body["data"]["serial_id"].as_i64().ok_or("wh serial")?;

            let inv_code = acct(&ctx, &["110701", "110700"])?;
            let (s, body) = ctx
                .client
                .post(
                    "/api/inventory/create/item",
                    json!({
                        "sku": "RM-WOOD-1",
                        "name": "Wood Plank",
                        "warehouse_serial": wh,
                        "unit": "pc",
                        "selling_price": "0",
                        "standard_cost": "5000",
                        "track_quantity": true,
                        "reorder_level": "100",
                        "asset_account": inv_code,
                        "cogs_account": "510100",
                        "income_account": "410100"
                    }),
                )
                .await
                .map_err(|e| e.to_string())?;
            assert!(ok(s), "create RM: {s} {body}");
            let uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
            let serial = body["data"]["serial_id"].as_i64().ok_or("rm serial")?;
            println!("✓ created RM serial={serial}");
            (uuid, serial, wh)
        };
    let _ = rm_serial; // available if single-issue path needed later

    // ----- Finished goods item -----
    let inv_code = acct(&ctx, &["110702", "110701", "110700"])?;
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/item",
            json!({
                "sku": "FG-CHAIR-1",
                "name": "Wooden Chair",
                "warehouse_serial": warehouse_serial,
                "unit": "pc",
                "selling_price": "45000",
                "standard_cost": "25000",
                "track_quantity": true,
                "reorder_level": "5",
                "asset_account": inv_code,
                "cogs_account": "510100",
                "income_account": "410100"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create FG: {s} {body}");
    let fg_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ FG uuid={fg_uuid}");

    // ----- 1. BOM: 1 chair ← 5 wood planks -----
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-CHAIR-v1",
                    "product_item_uuid": fg_uuid,
                    "description": "Standard wooden chair",
                    "revision": "1",
                    "is_default": true,
                    "is_active": true
                },
                "lines": [{
                    "line_number": 1,
                    "component_item_uuid": rm_uuid,
                    "quantity_per": "5",
                    "uom": "pc",
                    "scrap_factor": "1.0500"
                }]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create BOM: {s} {body}");
    let bom_uuid = extract_uuid(&body, &["data", "header", "uuid"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ BOM {bom_uuid}");

    // ----- 2. Production order -----
    let order_no = format!("PO-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "10",
                "start_date": "2025-01-15",
                "expected_completion_date": "2025-01-25"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create production order: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ production order {order_no} uuid={po_uuid}");

    // ----- 3. Bulk issue materials (BOM explode → Dr WIP / Cr RM) -----
    let wip = acct(&ctx, &["110703", "110750", "120300", "110700"])?;
    let raw = acct(&ctx, &["110701", "110700"])?;
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": warehouse_serial,
                "production_order_uuid": po_uuid,
                "wip_account_code": wip,
                "raw_mat_account_code": raw
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue materials: {s} {body}");
    println!("✓ materials issued to WIP");

    // ----- 4. Complete production (Dr FG / Cr WIP + variance accounts) -----
    let fg_acct = acct(&ctx, &["110703"])?;
    let mat_var = acct(&ctx, &["550100"])?;
    let lab_var = acct(&ctx, &["550200"])?;
    let moh_var = acct(&ctx, &["550300"])?;
    let control = acct(&ctx, &["550400"])?;

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/complete-production",
            json!({
                "production_order_uuid": po_uuid,
                "completed_quantity": "10",
                "wip_account_code": wip,
                "fg_account_code": fg_acct,
                "mfg_mat_var_code": mat_var,
                "mfg_lab_var_code": lab_var,
                "mfg_moh_var_code": moh_var,
                "control_code": control
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "complete production: {s} {body}");
    println!("✓ production completed → FG");

    // ----- 5. Trial balance -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after manufacturing");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ manufacturing_flow OK for {}", ctx.company_name);
    Ok(())
}
