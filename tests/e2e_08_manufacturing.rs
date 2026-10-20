mod common;

use common::client::extract_uuid;
use common::fixtures::{RMS, FG_CARGO_MULE, SA_CTRL_CORE, SA_PROP_GROUND};
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

let sa_ctrl = extract_uuid(find_item(&inv, SA_CTRL_CORE)?, &["uuid"]).map_err(|e| e.to_string())?;
let sa_prop = extract_uuid(find_item(&inv, SA_PROP_GROUND)?, &["uuid"]).map_err(|e| e.to_string())?;
    
// after find_item + RM uuid locals...
let fg_uuid = extract_uuid(find_item(&inv, FG_CARGO_MULE)?, &["uuid"])
    .map_err(|e| e.to_string())?;
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
                "routing_code": "RT-CARGO-MULE-v1",
                "notes": "Cargo Mule routing",
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
    // 4. BOM: CargoMule from electronics RM leaves (flat; multilevel later)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-CARGO-MULE-v1",
                    "product_item_uuid": fg_uuid,
                    "description": "CargoMule site delivery robot",
                    "revision": "1",
                    "is_default": true,
                    "is_active": true
                },
                "lines": [
                    {
                        "line_number": 1,
                        "component_item_uuid": mcues,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 2,
                        "component_item_uuid": imu,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 3,
                        "component_item_uuid": pcb,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 4,
                        "component_item_uuid": mtr,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 5,
                        "component_item_uuid": esc,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 6,
                        "component_item_uuid": frm,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 7,
                        "component_item_uuid": bat,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 8,
                        "component_item_uuid": sen,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 9,
                        "component_item_uuid": cbl,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 10,
                        "component_item_uuid": fst,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    }
                ]
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
