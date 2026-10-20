mod common;

use common::client::extract_uuid;
use common::fixtures::{RmSpec, RMS, FG_CARGO_MULE};
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

async fn uuid_by_sku(ctx: &TestContext, sku: &str) -> Result<String, String> {
    let (s, body) = ctx.client.get("/api/inventory/list/inventory").await.map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    body["data"].as_array().unwrap().iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .and_then(|i| i["uuid"].as_str())
        .map(|u| u.to_string())
        .ok_or_else(|| format!("sku {sku} not found — run e2e_05"))
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

   /* let (rm_uuid, rm_serial, warehouse_serial) =
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


// Find item uuid + serial + warehouse by sku from list/inventory payload.
fn find_item<'a>(
    inv: &'a serde_json::Value,
    sku: &str,
) -> Result<(&'a serde_json::Value), String> {
    inv["data"]
        .as_array()
        .ok_or_else(|| "inventory data not an array".into())?
        .iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .ok_or_else(|| format!("sku {sku} not in inventory — run e2e_05 first"))
}*/

fn find_item<'a>(
    inv: &'a serde_json::Value,
    sku: &str,
) -> Result<&'a serde_json::Value, String> {
    inv["data"]
        .as_array()
        .ok_or_else(|| String::from("inventory data not an array"))?
        .iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .ok_or_else(|| format!("sku {sku} not in inventory — run e2e_05 first"))
}

// ----- Inventory baseline: RMs from P2P -----
let (s, inv) = ctx
    .client
    .get("/api/inventory/list/inventory")
    .await
    .map_err(|e| e.to_string())?;
assert!(ok(s), "list inventory: {s} {inv}");

// Require the shared catalog (not "first row")
for rm in RMS {
    let _ = find_item(&inv, rm.sku)?;
}
println!("✓ all {} RM SKUs present", RMS.len());

// Warehouse: from any known RM (same WH-RM from e2e_05)
let anchor = find_item(&inv, RMS[0].sku)?;
let warehouse_serial = anchor["warehouse_serial"]
    .as_i64()
    .or_else(|| anchor["warehouse_serial_id"].as_i64())
    .unwrap_or(1);

// UUIDs you need for BOM lines
let mcues = extract_uuid(find_item(&inv, "MCU-STM32")?, &["uuid"])?;
let imu = extract_uuid(find_item(&inv, "IMU-9DOF")?, &["uuid"])?;
let mcust = extract_uuid(find_item(&inv, "MCU-ESP32")?, &["uuid"])?;
let esc = extract_uuid(find_item(&inv, "ESC-40A")?, &["uuid"])?;
let mtr = extract_uuid(find_item(&inv, "MTR-BLDC")?, &["uuid"])?;
let bat = extract_uuid(find_item(&inv, "BAT-4S5AH")?, &["uuid"])?;
let pcb = extract_uuid(find_item(&inv, "PCB-CTRL-V1")?, &["uuid"])?;
let frm = extract_uuid(find_item(&inv, "FRM-GROUND")?, &["uuid"])?;
let sen = extract_uuid(find_item(&inv, "SENS-US")?, &["uuid"])?;
let cbl = extract_uuid(find_item(&inv, "CBL-HARN")?, &["uuid"])?;
let fst = extract_uuid(find_item(&inv, "FST-KIT")?, &["uuid"])?;
/*
    // ----- Finished goods item -----
    let inv_code = acct(&ctx, &["110703"])?;
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/item",
            json!({
                "sku": "FG-CARGO-MULE",
                "name": "Cargo Mule",
                "warehouse_serial": warehouse_serial,
                "unit": "pc",
                "selling_price": "45000",
                "standard_cost": "25000",
                "track_quantity": true,
                "reorder_level": "5",
                "asset_account": inv_code,
                "cogs_account": "500000",
                "income_account": "410000"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create FG: {s} {body}");*/
    let fg_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ FG uuid={fg_uuid}");

    // =========================================================================
    // 1. Work centers (labor rate + allocation base for OH)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/workcenter",
            json!({
                "code": "WC-ASSY",
                "name": "Assembly Bench",
                "labor_rate": "15000",          // UGX per hour
                "allocation_base": "DirectLaborHours",
                "department_code": "Production"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "work center assembly: {s} {body}");
    println!("✓ work center WC-ASSY");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/workcenter",
            json!({
                "code": "WC-FINISH",
                "name": "Finishing Station",
                "labor_rate": "12000",
                "allocation_base": "MachineHours",
                "department_code": "Production"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "work center finish: {s} {body}");
    println!("✓ work center WC-FINISH");

    // =========================================================================
    // 2. Routing header + operations (sequence on work centers)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/header",
            json!({
                "product_item_uuid": fg_uuid,
                "routing_code": "RT-CHAIR-v1",
                "notes": "Standard chair routing",
                "version": "1",
                "base_quantity": 1,
                "effective_date": "2025-01-01",
                "status": "active",
                "is_default": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "routing header: {s} {body}");
    let routing_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ routing {routing_uuid}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/operation",
            json!({
                "routing_uuid": routing_uuid,
                "sequence": 10,
                "operation_name": "Assemble frame",
                "work_center": "WC-ASSY",
                "description": "Join legs and seat",
                "setup_time_minutes": "15",
                "run_time_minutes": "30"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "op 10: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/operation",
            json!({
                "routing_uuid": routing_uuid,
                "sequence": 20,
                "operation_name": "Sand and finish",
                "work_center": "WC-FINISH",
                "description": "Surface finish",
                "setup_time_minutes": "10",
                "run_time_minutes": "20"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "op 20: {s} {body}");
    println!("✓ routing operations 10, 20");

    // =========================================================================
    // 3. Overhead rate (predetermined OH for the period)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/overhead-rate",
            json!({
                "period_start": "2025-01-01",
                "period_end": "2025-12-31",
                "allocation_base": "DirectLaborHours",
                "estimated_overhead": "12000000",  // budgeted MOH
                "estimated_base": "800",           // budgeted labor hours
                "department_code": "Production",
                "is_active": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "overhead rate: {s} {body}");
    // rate ≈ 15000 UGX per labor hour
    println!("✓ overhead rate LABOR_HOURS");

    // =========================================================================
    // 4. BOM: 1 chair ← 5 wood planks
    // =========================================================================
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

    // =========================================================================
    // 5. Production order
    // =========================================================================
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

    // =========================================================================
    // 6. Bulk issue materials (BOM explode → Dr WIP / Cr RM)
    // =========================================================================
    let wip = acct(&ctx, &["110702"])?;
    let raw = acct(&ctx, &["110701"])?;
    let labor_payable = acct(&ctx, &["210300"])?;
    let oh_control = acct(&ctx, &["550400"])?;
    let cash_or_ap = acct(&ctx, &["210100"])?;

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

    // =========================================================================
    // 7. Apply direct labor (Dr WIP / Cr wages payable)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({
                "production_order_uuid": po_uuid,
                "labor_account_code": labor_payable,
                "wip_account_code": wip,
                "reference": "DL-ASSY-FINISH-10chairs"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply direct labor: {s} {body}");
    println!("✓ direct labor applied");

    // =========================================================================
    // 8. Apply predetermined overhead (rate × base hours)
    //    Note: DTO uses production_order as order_number string, not uuid
    // =========================================================================
    // 10 chairs × (30+20)/60 h run ≈ 8.3 labor hours; use 10 for clean math
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/applied",
            json!({
                "production_order": order_no,
                "base_hours": "10",
                "wip_account": wip,
                "overhead_control_account": oh_control,
                "allocation_base": "DirectLaborHours"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply overhead: {s} {body}");
    println!("✓ applied overhead (predetermined)");

    // =========================================================================
    // 9. Recognize actual overhead paid (optional — creates under/over applied)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/actual",
            json!({
                "production_order_uuid": po_uuid,
                "base_amount": "140000",
                "payable_or_cash": cash_or_ap,
                "overhead_control_account": oh_control,
                "allocation_base": "DirectLaborHours",
                "reference": "ACT-OH-JAN-UTILITIES"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "actual overhead: {s} {body}");
    println!("✓ actual overhead recognized");

    // =========================================================================
    // 7. Complete production (Dr FG / Cr WIP + variance accounts)
    // =========================================================================
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

    // =========================================================================
    // 8. Trial balance
    // =========================================================================
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after manufacturing");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ manufacturing_flow OK for {}", ctx.company_name);
    Ok(())
}
