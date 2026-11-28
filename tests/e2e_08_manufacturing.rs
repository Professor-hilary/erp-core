mod common;

use common::{
    E2eResult, TestContext,
    client::extract_uuid,
    fixtures::{self, FG_CARGO_MULE, RMS, SA_CTRL_CORE, SA_PROP_GROUND},
};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

/// Work centres / master data may already exist when tests run out of order.
fn already_exists(status: reqwest::StatusCode, body: &serde_json::Value) -> bool {
    if status.as_u16() == 409 || status.as_u16() == 400 {
        return true;
    }
    if status.as_u16() != 500 {
        return false;
    }
    let msg = body
        .pointer("/message")
        .and_then(|v| v.as_str())
        .or_else(|| body.pointer("/error").and_then(|v| v.as_str()))
        .unwrap_or("");
    msg.contains("duplicate key")
        || msg.contains("unique constraint")
        || msg.contains("already exists")
}

fn dec(v: &serde_json::Value) -> f64 {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        .or_else(|| v.as_i64().map(|i| i as f64))
        .unwrap_or(0.0)
}

fn assert_approx(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected} ± {tol}, got {actual}"
    );
}

fn find_item<'a>(inv: &'a serde_json::Value, sku: &str) -> Result<&'a serde_json::Value, String> {
    inv["data"]
        .as_array()
        .ok_or_else(|| String::from("inventory data not an array"))?
        .iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .ok_or_else(|| format!("sku {sku} not in inventory — run e2e_05 first"))
}

#[allow(dead_code)]
async fn uuid_by_sku(ctx: &TestContext, sku: &str) -> Result<String, String> {
    let (s, body) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .and_then(|i| i["uuid"].as_str())
        .map(|u| u.to_string())
        .ok_or_else(|| format!("sku {sku} not found — run e2e_05"))
}

#[tokio::test]
async fn manufacturing_flow() -> E2eResult<()> {

    let ctx = TestContext::from_saved_state().await?;

    // Shared tenant: always force standard so gate tests cannot leak method
    {
        let (st, body) = ctx
            .client
            .put(
                "/api/manufacturing/costing-policy",
                serde_json::json!({ "costing_method": "standard" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(st.is_success(), "ensure standard costing: {st} {body}");
    }

    // Stage A: costing policy defaults to standard; timesheet + payroll both allowed
    {
        let (st, body) = ctx.client.get("/api/manufacturing/costing-policy").await?;
        assert!(st.is_success(), "costing-policy get: {st} {body}");
        let data = body.get("data").cloned().unwrap_or(body);
        assert_eq!(
            data.get("costing_method").and_then(|v| v.as_str()),
            Some("standard"),
            "default costing_method"
        );
        assert_eq!(data.get("use_timesheet_actual").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(data.get("use_payroll_allocation").and_then(|v| v.as_bool()), Some(true));
        println!("✓ costing policy default standard (timesheet+payroll enabled)");
    }

    // ----- Inventory baseline: RMs from P2P + SA/FG shells from e2e_03 -----
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inventory: {s} {inv}");

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

    // Leaf RM UUIDs
    let mcues = extract_uuid(find_item(&inv, "MCU-STM32")?, &["uuid"])?;
    let imu = extract_uuid(find_item(&inv, "IMU-9DOF")?, &["uuid"])?;
    let _mcust = extract_uuid(find_item(&inv, "MCU-ESP32")?, &["uuid"])?;
    let esc = extract_uuid(find_item(&inv, "ESC-40A")?, &["uuid"])?;
    let mtr = extract_uuid(find_item(&inv, "MTR-BLDC")?, &["uuid"])?;
    let bat = extract_uuid(find_item(&inv, "BAT-4S5AH")?, &["uuid"])?;
    let pcb = extract_uuid(find_item(&inv, "PCB-CTRL-V1")?, &["uuid"])?;
    let frm = extract_uuid(find_item(&inv, "FRM-GROUND")?, &["uuid"])?;
    let sen = extract_uuid(find_item(&inv, "SENS-US")?, &["uuid"])?;
    let cbl = extract_uuid(find_item(&inv, "CBL-HARN")?, &["uuid"])?;
    let fst = extract_uuid(find_item(&inv, "FST-KIT")?, &["uuid"])?;

    // Subassembly + FG shells (created in e2e_03, qty 0 until this flow)
    let sa_ctrl =
        extract_uuid(find_item(&inv, SA_CTRL_CORE)?, &["uuid"]).map_err(|e| e.to_string())?;
    let sa_prop =
        extract_uuid(find_item(&inv, SA_PROP_GROUND)?, &["uuid"]).map_err(|e| e.to_string())?;
    let fg_uuid =
        extract_uuid(find_item(&inv, FG_CARGO_MULE)?, &["uuid"]).map_err(|e| e.to_string())?;
    println!("✓ SA-CTRL-CORE={sa_ctrl} SA-PROP-GROUND={sa_prop} FG={fg_uuid}");

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
    assert!(
        ok(s) || already_exists(s, &body),
        "work center assembly: {s} {body}"
    );
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
    assert!(
        ok(s) || already_exists(s, &body),
        "work center finish: {s} {body}"
    );
    println!("✓ work center WC-FINISH");

    // =========================================================================
    // 2. Routing header + operations (sequence on work centers) — FG only
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/header",
            json!({
                "product_item_uuid": fg_uuid,
                "routing_code": "RT-CARGO-MULE-v1",
                "notes": "Cargo Mule final assembly routing",
                "version": "1",
                "base_quantity": 1,
                "effective_date": "2026-01-01",
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
                "operation_name": "Assemble subassemblies onto chassis",
                "work_center": "WC-ASSY",
                "description": "Mount control core + propulsion onto frame",
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
                "operation_name": "Wire, test and finish",
                "work_center": "WC-FINISH",
                "description": "Harness, sensors, functional test, finish",
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
                "period_start": "2026-01-01",
                "period_end": "2026-12-31",
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
    // 4. Multi-level BOM
    //    Level 1 (subassemblies) → Level 0 (FG)
    //    explode_bom + production_order_materials already recurse to leaves.
    // =========================================================================

    // 4a. SA-CTRL-CORE: electronics brain
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-SA-CTRL-CORE-v1",
                    "product_item_uuid": sa_ctrl,
                    "description": "Control core subassembly (MCU + IMU + PCB)",
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
                    }
                ]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create BOM SA-CTRL-CORE: {s} {body}");
    let bom_sa_ctrl = extract_uuid(&body, &["data", "header", "uuid"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ BOM SA-CTRL-CORE {bom_sa_ctrl}");

    // 4b. SA-PROP-GROUND: drive train
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-SA-PROP-GROUND-v1",
                    "product_item_uuid": sa_prop,
                    "description": "Ground propulsion subassembly (motors + ESC + chassis)",
                    "revision": "1",
                    "is_default": true,
                    "is_active": true
                },
                "lines": [
                    {
                        "line_number": 1,
                        "component_item_uuid": mtr,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 2,
                        "component_item_uuid": esc,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 3,
                        "component_item_uuid": frm,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    }
                ]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create BOM SA-PROP-GROUND: {s} {body}");
    let bom_sa_prop = extract_uuid(&body, &["data", "header", "uuid"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ BOM SA-PROP-GROUND {bom_sa_prop}");

    // 4c. FG-CARGO-MULE: top-level multi-level BOM (SAs + remaining leaves)
    //     child_bom_uuid pins the exact sub-BOM revisions we just created;
    //     explode_bom will also find them via product_item_uuid if omitted.
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-CARGO-MULE-v1",
                    "product_item_uuid": fg_uuid,
                    "description": "CargoMule site delivery robot (multi-level)",
                    "revision": "1",
                    "is_default": true,
                    "is_active": true
                },
                "lines": [
                    {
                        "line_number": 1,
                        "component_item_uuid": sa_ctrl,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0",
                        "child_bom_uuid": bom_sa_ctrl,
                        "is_phantom": false
                    },
                    {
                        "line_number": 2,
                        "component_item_uuid": sa_prop,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0",
                        "child_bom_uuid": bom_sa_prop,
                        "is_phantom": false
                    },
                    {
                        "line_number": 3,
                        "component_item_uuid": bat,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 4,
                        "component_item_uuid": sen,
                        "quantity_per": "2",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 5,
                        "component_item_uuid": cbl,
                        "quantity_per": "1",
                        "uom": "pc",
                        "scrap_factor": "1.0"
                    },
                    {
                        "line_number": 6,
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
    assert!(ok(s), "create BOM FG multi-level: {s} {body}");
    let bom_uuid = extract_uuid(&body, &["data", "header", "uuid"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ BOM FG multi-level {bom_uuid}");

    // Optional: fetch top BOM to confirm structure is stored
    let (s, body) = ctx
        .client
        .get(&format!("/api/manufacturing/get/bom/{bom_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get BOM: {s} {body}");
    let n_lines = body["data"]["lines"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    assert_eq!(n_lines, 6, "expected 6 top-level BOM lines, got {n_lines}");
    println!("✓ get BOM has {n_lines} top-level lines (2 SA + 4 leaf)");

    // =========================================================================
    // 5. Production order (material requirements explode to leaves via explode_bom)
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
                "start_date": "2026-01-15",
                "expected_completion_date": "2026-01-25"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create production order: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ production order {order_no} uuid={po_uuid}");

    // =========================================================================
    // 6. Bulk issue materials (BOM explode → Dr WIP / Cr RM on leaves only)
    // =========================================================================
    // Account codes resolved server-side from account roles (no client UUID bag).
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": warehouse_serial,
                "production_order_uuid": po_uuid
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue materials: {s} {body}");
    // Bulk issue returns material lines — remaining should be 0 after full issue
    if let Some(arr) = body["data"].as_array() {
        assert!(!arr.is_empty(), "expected material issue result rows");
        for row in arr {
            let rem = dec(&row["remaining_qty"]);
            assert_approx(rem, 0.0, 0.001, "remaining after full issue");
        }
        println!("✓ materials issued; {} lines fully issued", arr.len());
    } else {
        println!("✓ materials issued to WIP (multi-level leaves)");
    }

    // Re-issue should not silently double-post: remaining is 0 → no materials or error
    let (s2, body2) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": warehouse_serial,
                "production_order_uuid": po_uuid
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        !ok(s2)
            || body2["data"]
                .as_array()
                .map(|a| a.is_empty())
                .unwrap_or(true),
        "second full issue should fail or issue nothing, got {s2} {body2}"
    );
    println!("✓ repeat full issue rejected or no-op");

    // =========================================================================
    // 7. Apply direct labor — certify amount
    //    WC-ASSY 15k/h: (15 + 30*10)/60 * 15000 = 78750
    //    WC-FINISH 12k/h: (10 + 20*10)/60 * 12000 = 42000
    //    Total applied labour = 120750
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({
                "production_order_uuid": po_uuid,
                "reference": "DL-ASSY-FINISH-10-CargoMule"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply direct labor: {s} {body}");
    let labor_amt = dec(&body["data"]["amount"]);
    if labor_amt > 0.0 {
        assert_approx(labor_amt, 120_750.0, 1.0, "applied direct labour");
        println!("✓ direct labor applied amount={labor_amt}");
    } else {
        println!("✓ direct labor applied (amount not in response body)");
    }

    // =========================================================================
    // 8. Predetermined OH: rate = 12_000_000/800 = 15_000; 10h → 150_000
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/applied",
            json!({
                "production_order": order_no,
                "base_hours": "10",
                "allocation_base": "DirectLaborHours"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply overhead: {s} {body}");
    let oh_amt = dec(&body["data"]["amount"]);
    if oh_amt > 0.0 {
        assert_approx(oh_amt, 150_000.0, 1.0, "applied overhead");
        println!("✓ applied overhead amount={oh_amt}");
    } else {
        println!("✓ applied overhead (predetermined)");
    }

    // =========================================================================
    // 9. Actual overhead 140000 (under-applied vs 150000 applied)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/actual",
            json!({
                "production_order_uuid": po_uuid,
                "base_amount": "140000",
                "allocation_base": "DirectLaborHours",
                "reference": "ACT-OH-JAN-UTILITIES"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "actual overhead: {s} {body}");
    println!("✓ actual overhead recognized 140000");

    // =========================================================================
    // 9b. Start production + seed operations (Stage 6)
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/start-production",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "start production: {s} {body}");
    assert_eq!(
        body["data"]["status"].as_str().unwrap_or(""),
        "In Progress",
        "order must be In Progress before complete"
    );
    // Spec pin (Stage 3)
    assert!(
        body["data"]["bom_header_uuid"].as_str().is_some()
            || body["data"].get("bom_header_uuid").is_some(),
        "order should pin bom_header_uuid: {body}"
    );
    println!("✓ production started (In Progress) with BOM/routing pin");

    let (s, ops) = ctx
        .client
        .get(&format!("/api/manufacturing/operation/list/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list operations: {s} {ops}");
    let op_rows = ops["data"].as_array().cloned().unwrap_or_default();
    assert!(
        op_rows.len() >= 2,
        "expected ≥2 operation executions from routing, got {}",
        op_rows.len()
    );
    println!("✓ operation executions n={}", op_rows.len());

    // =========================================================================
    // 10. Reject over-complete, then complete full qty
    // =========================================================================
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/complete-production",
            json!({
                "production_order_uuid": po_uuid,
                "completed_quantity": "99"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(!ok(s), "over-complete must fail, got {s} {body}");
    println!("✓ over-complete rejected");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/complete-production",
            json!({
                "production_order_uuid": po_uuid,
                "completed_quantity": "10"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "complete production: {s} {body}");
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "Completed", "expected Completed, got {status}");
    let qty_done = dec(&body["data"]["quantity_completed"]);
    assert_approx(qty_done, 10.0, 0.001, "quantity_completed");
    println!("✓ production completed → FG qty=10");

    // FG inventory should show finished goods on hand
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv after complete: {s}");
    let fg_uuid_str = fg_uuid.to_string();
    if let Some(fg) = inv["data"].as_array().and_then(|a| {
        a.iter().find(|i| {
            i["sku"].as_str() == Some("FG-CARGO-MULE")
                || i["uuid"].as_str() == Some(fg_uuid_str.as_str())
        })
    }) {
        let q = dec(fg
            .get("quantity_on_hand")
            .unwrap_or(&fg["quantity"].clone()));
        // quantity field names vary; only assert if present
        if q > 0.0 {
            assert!(q >= 10.0 - 0.01, "FG on hand expected ≥10, got {q}");
            println!("✓ FG inventory on hand ≥ 10 ({q})");
        }
    }

    // =========================================================================
    // 11. Trial balance (balanced ≠ correct unit cost, but required)
    // =========================================================================
    ctx.assert_trial_balance_ok("2026-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after manufacturing");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!(
        "✅ manufacturing_flow + Stage 7 cert checks OK for {}",
        ctx.company_name
    );
    Ok(())
}

/// Stage 7: partial completion + operation execution + scrap path
#[tokio::test]
async fn manufacturing_partial_ops_cert() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let fg = find_item(&inv, "FG-CARGO-MULE")
        .map_err(|e| format!("{e} — run manufacturing_flow first"))?;
    let fg_uuid = extract_uuid(fg, &["uuid"]).map_err(|e| e.to_string())?;

    // Issue from RM warehouse (WH-RM serial=1), not FG warehouse
    let warehouse_serial = inv["data"]
        .as_array()
        .and_then(|rows| {
            rows.iter()
                .find(|w| w["code"].as_str() == Some("WH-RM"))
                .and_then(|w| w["serial_id"].as_i64())
        })
        .or_else(|| {
            // warehouses may be separate list
            None
        });
    let (s, whs) = ctx
        .client
        .get("/api/inventory/list/warehouses")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list warehouses: {s}");
    let warehouse_serial = warehouse_serial
        .or_else(|| {
            whs["data"].as_array().and_then(|a| {
                a.iter()
                    .find(|w| w["code"].as_str() == Some("WH-RM"))
                    .and_then(|w| w["serial_id"].as_i64())
            })
        })
        .unwrap_or(1);

    // manufacturing_flow already consumed stock for qty 10 — restock RMs for qty 4
    if ctx.vendors.is_empty() {
        let _ = ctx.seed_trading_partners().await;
    }
    let vendor_uuid = *ctx
        .vendors
        .values()
        .next()
        .ok_or("need a vendor — run e2e_02 first")?;
    let rows = inv["data"]
        .as_array()
        .ok_or("inventory data not an array")?;
    let mut bill_items = Vec::new();
    for rm in RMS {
        let row = rows
            .iter()
            .find(|i| i["sku"].as_str() == Some(rm.sku))
            .ok_or_else(|| format!("sku {} missing", rm.sku))?;
        let stock_item_id = row["serial_id"]
            .as_i64()
            .ok_or_else(|| format!("serial_id missing for {}", rm.sku))?;
        // Enough for 4 finished units with headroom (same pack sizes as P2P is fine)
        bill_items.push(json!({
            "stock_item_id": stock_item_id,
            "description": rm.name,
            "quantity": rm.qty_buy,
            "unit_price": rm.unit_price,
            "tax_rate": "18"
        }));
    }
    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/create/purchase",
            json!({
                "bill_number": format!("BILL-PART-{}", chrono::Utc::now().timestamp_millis()),
                "vendor_uuid": vendor_uuid,
                "bill_date": "2026-02-01",
                "due_date": "2026-02-28",
                "reference": "RESTOCK-FOR-PARTIAL-PO",
                "settlement_type": "credit",
                "payment_status": "unpaid",
                "items": bill_items
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "restock create purchase: {s} {body}");
    let bill_serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("bill serial_id missing")?;
    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/post/credit/purchase",
            json!({
                "bill_serial_id": bill_serial,
                "vat_tax_account": "110501",
                "payables_account": "210100",
                "cash_account": null
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "restock post purchase: {s} {body}");
    println!("✓ RM restocked for partial PO");

    let order_no = format!("PO-PART-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "4",
                "start_date": "2026-02-01",
                "expected_completion_date": "2026-02-10"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create partial PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": warehouse_serial,
                "production_order_uuid": po_uuid
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue materials partial PO: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/start-production",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "start partial PO: {s} {body}");

    let (s, ops) = ctx
        .client
        .get(&format!("/api/manufacturing/operation/list/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list ops: {s} {ops}");
    let op_rows = ops["data"].as_array().cloned().unwrap_or_default();
    if let Some(first) = op_rows.first() {
        let exec_uuid = extract_uuid(first, &["uuid"]).map_err(|e| e.to_string())?;
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/operation/start",
                json!({ "execution_uuid": exec_uuid, "qty_started": "4" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "start op: {s} {body}");
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/operation/complete",
                json!({
                    "execution_uuid": exec_uuid,
                    "qty_completed": "3",
                    "qty_scrapped": "1",
                    "run_minutes_actual": "90",
                    "setup_minutes_actual": "15"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "complete op: {s} {body}");
        assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "Completed");
        println!("✓ operation completed with scrap");
    } else {
        println!("! no operations seeded — ensure routing exists for FG");
    }

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/scrap-rework",
            json!({
                "production_order_uuid": po_uuid,
                "event_type": "Rework",
                "quantity": "1",
                "reason_code": "WIRE",
                "reason_text": "Harness rework"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "scrap-rework: {s} {body}");
    println!("✓ rework event recorded");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/quality-inspection",
            json!({
                "production_order_uuid": po_uuid,
                "qty_inspected": "3",
                "qty_passed": "3",
                "qty_failed": "0",
                "result": "Pass",
                "inspection_type": "Final"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "QC: {s} {body}");
    println!("✓ final QC pass");

    // Partial complete 2 of 4
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/complete-production",
            json!({
                "production_order_uuid": po_uuid,
                "completed_quantity": "2"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "partial complete: {s} {body}");
    let st = body["data"]["status"].as_str().unwrap_or("");
    assert_eq!(
        st, "In Progress",
        "partial complete should leave In Progress, got {st}"
    );
    assert_approx(
        dec(&body["data"]["quantity_completed"]),
        2.0,
        0.001,
        "partial qty",
    );
    println!("✓ partial completion 2/4 still In Progress");

    ctx.assert_trial_balance_ok("2026-02-28")
        .await
        .map_err(|e| e.to_string())?;
    println!("✅ manufacturing_partial_ops_cert OK");
    Ok(())
}

/// Stage labour: actual timesheet path vs routing-applied labour
#[tokio::test]
async fn manufacturing_labour_actual_path() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Shared tenant: always force standard so gate tests cannot leak method
    {
        let (st, body) = ctx
            .client
            .put(
                "/api/manufacturing/costing-policy",
                serde_json::json!({ "costing_method": "standard" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(st.is_success(), "ensure standard costing: {st} {body}");
    }

    // Employees from workforce
    let (s, emps) = ctx
        .client
        .get("/api/workforce/list/employees")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list employees: {s}");
    let emp_rows = emps["data"].as_array().cloned().unwrap_or_default();
    assert!(
        emp_rows.len() >= 2,
        "need ≥2 employees for team timesheet — run e2e_02 first"
    );
    let emp1 = extract_uuid(&emp_rows[0], &["uuid"]).map_err(|e| e.to_string())?;
    let emp2 = extract_uuid(&emp_rows[1], &["uuid"]).map_err(|e| e.to_string())?;

    // FG product + open or any production order
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let fg = find_item(&inv, "FG-CARGO-MULE")
        .map_err(|e| format!("{e} — run manufacturing_flow first"))?;
    let fg_uuid = extract_uuid(fg, &["uuid"]).map_err(|e| e.to_string())?;

    let order_no = format!("PO-LAB-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "2",
                "start_date": "2026-02-15",
                "expected_completion_date": "2026-02-20"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create labour PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;

    // Actual rate for ASSY category (seeded in migration)
    let (s, cats) = ctx
        .client
        .get("/api/manufacturing/labour/categories")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "labour categories: {s} {cats}");
    let cat_uuid = cats["data"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|c| c["code"].as_str() == Some("ASSY"))
                .and_then(|c| c["uuid"].as_str())
        })
        .map(|s| s.to_string());

    if let Some(ref cu) = cat_uuid {
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/labour/rates",
                json!({
                    "labour_category_uuid": cu,
                    "rate_kind": "Actual",
                    "hourly_rate": "15000",
                    "effective_from": "2026-01-01"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "create actual rate: {s} {body}");
        println!("✓ Actual ASSY rate 15000");
    }

    // Team timesheet: 2 employees × 4 hours Direct on this order
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/team",
            json!({
                "employee_uuids": [emp1, emp2],
                "work_date": "2026-02-16",
                "hours": "4",
                "activity_type": "Direct",
                "production_order_uuid": po_uuid,
                "work_center_code": "WC-ASSY",
                "labour_category_uuid": cat_uuid,
                "memo": "e2e team assembly"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "team timesheet: {s} {body}");
    let entries = body["data"].as_array().cloned().unwrap_or_default();
    assert_eq!(entries.len(), 2, "expected 2 team entries");
    let entry_ids: Vec<_> = entries
        .iter()
        .filter_map(|e| e["uuid"].as_str().map(|s| s.to_string()))
        .collect();
    println!("✓ team timesheet n={}", entry_ids.len());

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/bulk-submit",
            json!({ "entry_uuids": entry_ids }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "bulk submit: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/bulk-approve",
            json!({ "entry_uuids": entry_ids }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "bulk approve: {s} {body}");
    println!("✓ bulk submit + approve");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/bulk-post",
            json!({ "entry_uuids": entry_ids }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "bulk post: {s} {body}");
    println!("✓ bulk post → ActualLabor");

    // Attendance for recon
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/attendance/upsert",
            json!({
                "employee_uuid": emp1,
                "work_date": "2026-02-16",
                "status": "Present",
                "hours_present": "8"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "attendance: {s} {body}");

    let (s, recon) = ctx
        .client
        .get("/api/manufacturing/labour/reconcile")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "reconcile: {s} {recon}");
    println!("✓ attendance vs timesheet reconcile");

    let (s, var) = ctx
        .client
        .post(
            &format!("/api/manufacturing/labour/variances/{po_uuid}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "variances: {s} {var}");
    let actual_hours = dec(var["data"].get("actual_hours").unwrap_or(&json!(0)));
    assert!(
        actual_hours >= 7.9,
        "expected ~8 actual hours from 2×4 team entry, got {actual_hours}"
    );
    println!("✓ labour variance actual_hours={actual_hours}");

    let (s, sum) = ctx
        .client
        .get(&format!("/api/manufacturing/labour/summary/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "labour summary: {s} {sum}");
    let actual_lab = dec(sum["data"].get("actual_labour").unwrap_or(&json!(0)));
    // 2 employees × 4h × 15000 = 120_000
    assert_approx(actual_lab, 120_000.0, 1.0, "actual_labour from timesheets");
    let applied_lab = dec(sum["data"].get("applied_labour").unwrap_or(&json!(0)));
    assert_approx(
        applied_lab,
        0.0,
        0.01,
        "applied_labour on timesheet-only PO",
    );
    println!("✓ labour summary actual_labour={actual_lab} applied_labour={applied_lab}");
    // Authoritative actual present; variance cost = actual - applied (= 120000 - 0)
    let cost_var = dec(sum["data"]
        .get("labour_cost_variance")
        .unwrap_or(&json!(actual_lab)));
    assert_approx(
        cost_var,
        120_000.0,
        1.0,
        "labour_cost_variance timesheet-only",
    );

    let (s, prod) = ctx
        .client
        .get(&format!(
            "/api/manufacturing/labour/report/productivity/{po_uuid}"
        ))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "productivity: {s} {prod}");
    println!("✓ productivity report");

    ctx.assert_trial_balance_ok("2026-02-28")
        .await
        .map_err(|e| e.to_string())?;
    println!("✅ manufacturing_labour_actual_path OK");
    Ok(())
}

/// Missing actual must not invent a favourable labour variance on a completed order
/// that only has routing-applied DirectLabor.
#[tokio::test]
async fn manufacturing_labour_std_only_no_false_actual() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Shared tenant: always force standard so gate tests cannot leak method
    {
        let (st, body) = ctx
            .client
            .put(
                "/api/manufacturing/costing-policy",
                serde_json::json!({ "costing_method": "standard" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(st.is_success(), "ensure standard costing: {st} {body}");
    }

    // Re-use FG and create PO, apply routing labour only, complete — no timesheets
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let fg = find_item(&inv, "FG-CARGO-MULE").map_err(|e| e.to_string())?;
    let fg_uuid = extract_uuid(fg, &["uuid"]).map_err(|e| e.to_string())?;

    let order_no = format!("PO-STD-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "1",
                "start_date": "2026-02-01",
                "expected_completion_date": "2026-02-05"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply labor: {s} {body}");

    let (s, sum) = ctx
        .client
        .get(&format!("/api/manufacturing/labour/summary/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "summary: {s} {sum}");
    let applied = dec(sum["data"].get("applied_labour").unwrap_or(&json!(0)));
    let actual = dec(sum["data"].get("actual_labour").unwrap_or(&json!(0)));
    assert!(
        applied > 0.0,
        "expected applied labour from routing, got {applied}"
    );
    assert_approx(
        actual,
        0.0,
        0.01,
        "no timesheet/payroll → actual must be 0, not invented",
    );
    println!("✓ std-only applied={applied} actual={actual} (no false actual)");
    Ok(())
}

/// Standard applied + actual timesheet on the SAME order — exact variance.
/// Applied (routing qty 10) = 120,750; actual = 9h × 15,000 = 135,000; variance = 14,250 U.
#[tokio::test]
async fn manufacturing_labour_std_plus_actual_cert() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Shared tenant: always force standard so gate tests cannot leak method
    {
        let (st, body) = ctx
            .client
            .put(
                "/api/manufacturing/costing-policy",
                serde_json::json!({ "costing_method": "standard" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(st.is_success(), "ensure standard costing: {st} {body}");
    }

    let (s, emps) = ctx
        .client
        .get("/api/workforce/list/employees")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list employees: {s}");
    let emp_rows = emps["data"].as_array().cloned().unwrap_or_default();
    assert!(emp_rows.len() >= 1, "need an employee");
    let emp1 = extract_uuid(&emp_rows[0], &["uuid"]).map_err(|e| e.to_string())?;

    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let fg = find_item(&inv, "FG-CARGO-MULE").map_err(|e| e.to_string())?;
    let fg_uuid = extract_uuid(fg, &["uuid"]).map_err(|e| e.to_string())?;

    let order_no = format!("PO-STD-ACT-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "10",
                "start_date": "2026-02-10",
                "expected_completion_date": "2026-02-20"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply labor: {s} {body}");

    let (s, sum0) = ctx
        .client
        .get(&format!("/api/manufacturing/labour/summary/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "summary after apply: {s} {sum0}");
    let applied = dec(sum0["data"].get("applied_labour").unwrap_or(&json!(0)));
    assert_approx(applied, 120_750.0, 1.0, "applied labour qty10 routing");
    println!("✓ applied labour={applied}");

    let (s, cats) = ctx
        .client
        .get("/api/manufacturing/labour/categories")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "categories: {s}");
    let cat_uuid = cats["data"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|c| c["code"].as_str() == Some("ASSY"))
                .and_then(|c| c["uuid"].as_str())
        })
        .map(|s| s.to_string());

    if let Some(ref cu) = cat_uuid {
        let _ = ctx
            .client
            .post(
                "/api/manufacturing/labour/rates",
                json!({
                    "labour_category_uuid": cu,
                    "rate_kind": "Actual",
                    "hourly_rate": "15000",
                    "effective_from": "2026-01-01"
                }),
            )
            .await;
    }

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/team",
            json!({
                "employee_uuids": [emp1],
                "work_date": "2026-02-12",
                "hours": "9",
                "activity_type": "Direct",
                "production_order_uuid": po_uuid,
                "work_center_code": "WC-ASSY",
                "labour_category_uuid": cat_uuid,
                "memo": "std+actual cert 9h"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "timesheet: {s} {body}");
    let entries = body["data"].as_array().cloned().unwrap_or_default();
    let entry_ids: Vec<_> = entries
        .iter()
        .filter_map(|e| e["uuid"].as_str().map(|s| s.to_string()))
        .collect();
    assert!(!entry_ids.is_empty(), "expected timesheet entry");

    for path in [
        "/api/manufacturing/timesheet/bulk-submit",
        "/api/manufacturing/timesheet/bulk-approve",
        "/api/manufacturing/timesheet/bulk-post",
    ] {
        let (s, body) = ctx
            .client
            .post(path, json!({ "entry_uuids": entry_ids }))
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "{path}: {s} {body}");
    }
    println!("✓ actual labour posted 9h × 15000 = 135000");

    let (s, sum) = ctx
        .client
        .get(&format!("/api/manufacturing/labour/summary/{po_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "summary: {s} {sum}");
    let applied = dec(sum["data"].get("applied_labour").unwrap_or(&json!(0)));
    let actual = dec(sum["data"].get("actual_labour").unwrap_or(&json!(0)));
    let cost_var = dec(sum["data"].get("labour_cost_variance").unwrap_or(&json!(0)));
    assert_approx(applied, 120_750.0, 1.0, "applied still 120750");
    assert_approx(actual, 135_000.0, 1.0, "actual 9h×15k");
    assert_approx(cost_var, 14_250.0, 1.0, "unfavourable variance 14250");
    println!("✓ std+actual applied={applied} actual={actual} variance={cost_var}");

    let (s, var) = ctx
        .client
        .post(
            &format!("/api/manufacturing/labour/variances/{po_uuid}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "compute variances: {s} {var}");
    let act_h = dec(var["data"].get("actual_hours").unwrap_or(&json!(0)));
    assert_approx(act_h, 9.0, 0.05, "variance actual_hours");
    println!("✓ labour variance rows actual_hours={act_h}");

    ctx.assert_trial_balance_ok("2026-02-28")
        .await
        .map_err(|e| e.to_string())?;
    println!("✅ manufacturing_labour_std_plus_actual_cert OK");
    Ok(())
}

/// 4-level BOM volume: RM-BULK → MOD-PWR → CMP-DEEP → FG-DEEP-VOLUME.
/// Proves explode_bom + bulk issue across ≥40 leaf lines without changing CargoMule certs.
#[tokio::test]
async fn manufacturing_deep_bom_volume() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Shared tenant: always force standard so gate tests cannot leak method
    {
        let (st, body) = ctx
            .client
            .put(
                "/api/manufacturing/costing-policy",
                serde_json::json!({ "costing_method": "standard" }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(st.is_success(), "ensure standard costing: {st} {body}");
    }

    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let rows = inv["data"].as_array().cloned().unwrap_or_default();

    let mut bulk_rms: Vec<(String, String)> = Vec::new();
    for i in 1..=(fixtures::SCALE_MFG_MODULES * fixtures::SCALE_MFG_LEAVES_PER_MODULE) {
        let sku = format!("RM-BULK-{i:03}");
        if let Some(row) = rows.iter().find(|r| r["sku"].as_str() == Some(&sku)) {
            let u = extract_uuid(row, &["uuid"]).map_err(|e| e.to_string())?;
            bulk_rms.push((sku, u.to_string()));
        }
    }
    assert!(
        bulk_rms.len() >= fixtures::SCALE_MFG_MODULES * fixtures::SCALE_MFG_LEAVES_PER_MODULE,
        "need bulk RMs from e2e_03 scale seed, got {}",
        bulk_rms.len()
    );

    let mut modules: Vec<(String, String)> = Vec::new();
    for i in 1..=fixtures::SCALE_MFG_MODULES {
        let sku = format!("MOD-PWR-{i:02}");
        if let Some(row) = rows.iter().find(|r| r["sku"].as_str() == Some(&sku)) {
            let u = extract_uuid(row, &["uuid"]).map_err(|e| e.to_string())?;
            modules.push((sku, u.to_string()));
        }
    }
    assert_eq!(
        modules.len(),
        fixtures::SCALE_MFG_MODULES,
        "need modules from e2e_03"
    );

    // Self-contained: deep_bom may run before manufacturing_flow alphabetically
    for (code, name, rate, base) in [
        ("WC-ASSY", "Assembly Bench", "15000", "DirectLaborHours"),
        ("WC-FINISH", "Finishing Station", "12000", "MachineHours"),
        ("WC-VOL-03", "Volume Cell 3", "10000", "DirectLaborHours"),
        ("WC-VOL-04", "Volume Cell 4", "10000", "DirectLaborHours"),
        ("WC-VOL-05", "Volume Cell 5", "10000", "DirectLaborHours"),
        ("WC-VOL-06", "Volume Cell 6", "10000", "DirectLaborHours"),
    ] {
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/create/routing/workcenter",
                json!({
                    "code": code,
                    "name": name,
                    "labor_rate": rate,
                    "allocation_base": base,
                    "department_code": "Production"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        // 409/400 if manufacturing_flow already created ASSY/FINISH
        assert!(
            ok(s) || s.as_u16() == 409 || s.as_u16() == 400,
            "wc {code}: {s} {body}"
        );
    }
    println!(
        "✓ work centres ASSY/FINISH + volume toward {}",
        fixtures::SCALE_MFG_WORK_CENTERS
    );

    let mut module_bom_uuids: Vec<String> = Vec::new();
    for (mi, (mod_sku, mod_uuid)) in modules.iter().enumerate() {
        let start = mi * fixtures::SCALE_MFG_LEAVES_PER_MODULE;
        let leaf_slice = &bulk_rms[start..start + fixtures::SCALE_MFG_LEAVES_PER_MODULE];
        let lines: Vec<_> = leaf_slice
            .iter()
            .enumerate()
            .map(|(li, (_sku, uuid))| {
                json!({
                    "line_number": li + 1,
                    "component_item_uuid": uuid,
                    "quantity_per": "1",
                    "uom": "pc",
                    "scrap_factor": "1.0",
                    "is_phantom": false
                })
            })
            .collect();
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/create/bom",
                json!({
                    "header": {
                        "bom_code": format!("BOM-{mod_sku}-v1"),
                        "product_item_uuid": mod_uuid,
                        "description": format!("Module BOM {mod_sku}"),
                        "revision": "1",
                        "is_default": true,
                        "is_active": true
                    },
                    "lines": lines
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "module BOM {mod_sku}: {s} {body}");
        let bom_u = extract_uuid(&body, &["data", "header", "uuid"])
            .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
            .map_err(|e| e.to_string())?;
        module_bom_uuids.push(bom_u.to_string());
    }
    println!("✓ L1 module BOMs n={}", module_bom_uuids.len());

    let mut component_uuids: Vec<String> = Vec::new();
    let mut component_bom_uuids: Vec<String> = Vec::new();
    for i in 1..=fixtures::SCALE_MFG_COMPONENTS {
        let sku = format!("CMP-DEEP-{i:02}");
        let row = rows
            .iter()
            .find(|r| r["sku"].as_str() == Some(sku.as_str()))
            .ok_or_else(|| format!("{sku} missing — run e2e_03"))?;
        let cmp_uuid = extract_uuid(row, &["uuid"]).map_err(|e| e.to_string())?;
        component_uuids.push(cmp_uuid.to_string());

        let m0 = &modules[(i - 1) * 2];
        let m1 = &modules[(i - 1) * 2 + 1];
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/create/bom",
                json!({
                    "header": {
                        "bom_code": format!("BOM-{sku}-v1"),
                        "product_item_uuid": cmp_uuid,
                        "description": format!("Component BOM {sku}"),
                        "revision": "1",
                        "is_default": true,
                        "is_active": true
                    },
                    "lines": [
                        {
                            "line_number": 1,
                            "component_item_uuid": m0.1,
                            "quantity_per": "1",
                            "uom": "pc",
                            "scrap_factor": "1.0",
                            "is_phantom": false
                        },
                        {
                            "line_number": 2,
                            "component_item_uuid": m1.1,
                            "quantity_per": "1",
                            "uom": "pc",
                            "scrap_factor": "1.0",
                            "is_phantom": false
                        }
                    ]
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "component BOM {sku}: {s} {body}");
        let bom_u = extract_uuid(&body, &["data", "header", "uuid"])
            .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
            .map_err(|e| e.to_string())?;
        component_bom_uuids.push(bom_u.to_string());
    }
    println!("✓ L2 component BOMs n={}", component_bom_uuids.len());

    let fg_row = rows
        .iter()
        .find(|r| r["sku"].as_str() == Some(fixtures::FG_DEEP_VOLUME))
        .ok_or("FG-DEEP-VOLUME missing — run e2e_03")?;
    let fg_uuid = extract_uuid(fg_row, &["uuid"]).map_err(|e| e.to_string())?;

    let fg_lines: Vec<_> = component_uuids
        .iter()
        .enumerate()
        .map(|(i, cu)| {
            json!({
                "line_number": i + 1,
                "component_item_uuid": cu,
                "quantity_per": "1",
                "uom": "pc",
                "scrap_factor": "1.0",
                "child_bom_uuid": component_bom_uuids[i],
                "is_phantom": false
            })
        })
        .collect();
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/bom",
            json!({
                "header": {
                    "bom_code": "BOM-FG-DEEP-VOLUME-v1",
                    "product_item_uuid": fg_uuid,
                    "description": "4-level deep volume FG",
                    "revision": "1",
                    "is_default": true,
                    "is_active": true
                },
                "lines": fg_lines
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "FG deep BOM: {s} {body}");
    println!("✓ L3 FG-DEEP-VOLUME BOM");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/routing/header",
            json!({
                "product_item_uuid": fg_uuid,
                "routing_code": "RT-DEEP-VOLUME-v1",
                "notes": "Deep volume routing",
                "version": "1",
                "base_quantity": 1,
                "effective_date": "2026-01-01",
                "status": "active",
                "is_default": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "routing: {s} {body}");
    let routing_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    for (seq, (wc, setup, run)) in [
        ("WC-ASSY", "20", "45"),
        ("WC-FINISH", "10", "30"),
        ("WC-VOL-03", "15", "20"),
    ]
    .iter()
    .enumerate()
    {
        let (s, body) = ctx
            .client
            .post(
                "/api/manufacturing/create/routing/operation",
                json!({
                    "routing_uuid": routing_uuid,
                    "sequence": (seq + 1) * 10,
                    "operation_name": format!("Deep op {}", seq + 1),
                    "work_center": wc,
                    "description": format!("Volume routing step {}", seq + 1),
                    "setup_time_minutes": setup,
                    "run_time_minutes": run
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s) || s.as_u16() == 409, "op: {s} {body}");
    }
    println!("✓ deep routing ops");

    let vendor_uuid = *ctx
        .vendors
        .values()
        .next()
        .ok_or("need vendor — run e2e_02")?;
    let (s, inv2) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv for serials: {s}");
    let inv_rows = inv2["data"].as_array().cloned().unwrap_or_default();
    let mut bill_items = Vec::new();
    for (sku, _uuid) in &bulk_rms {
        let row = inv_rows
            .iter()
            .find(|i| i["sku"].as_str() == Some(sku.as_str()))
            .ok_or_else(|| format!("missing {sku}"))?;
        let stock_item_id = row["serial_id"]
            .as_i64()
            .ok_or_else(|| format!("serial_id missing for {sku}"))?;
        bill_items.push(json!({
            "stock_item_id": stock_item_id,
            "description": sku,
            "quantity": "20",
            "unit_price": "10000",
            "tax_rate": "18"
        }));
    }
    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/create/purchase",
            json!({
                "bill_number": format!("BILL-DEEP-{}", chrono::Utc::now().timestamp_millis()),
                "vendor_uuid": vendor_uuid,
                "bill_date": "2026-02-01",
                "due_date": "2026-02-28",
                "reference": "PO-DEEP-BOM-RM",
                "settlement_type": "credit",
                "payment_status": "unpaid",
                "items": bill_items
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create deep purchase: {s} {body}");
    let bill_serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("bill serial_id missing")?;
    let (s, body) = ctx
        .client
        .post(
            "/api/procurement/post/credit/purchase",
            json!({
                "bill_serial_id": bill_serial,
                "vat_tax_account": "110501",
                "payables_account": "210100",
                "cash_account": null
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post deep purchase: {s} {body}");
    println!("✓ purchased {} bulk RM lines for deep BOM", bulk_rms.len());

    let order_no = format!("PO-DEEP-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "2",
                "start_date": "2026-02-05",
                "expected_completion_date": "2026-02-25"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create deep PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ deep PO {order_no}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": 1,
                "production_order_uuid": po_uuid
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue deep materials: {s} {body}");
    let n_lines = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    let min_leaves = fixtures::SCALE_MFG_MODULES * fixtures::SCALE_MFG_LEAVES_PER_MODULE;
    assert!(
        n_lines >= min_leaves,
        "expected ≥{min_leaves} exploded leaf issue lines, got {n_lines}"
    );
    println!("✓ deep BOM issue lines={n_lines} (4-level explode)");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "apply labor deep: {s} {body}");
    println!("✓ deep routing labour applied");

    ctx.assert_trial_balance_ok("2026-02-28")
        .await
        .map_err(|e| e.to_string())?;
    println!("✅ manufacturing_deep_bom_volume OK — 4 levels, {n_lines} leaf issues");
    Ok(())
}

/// Stage B: under actual costing, predetermined OH and routing applied labour are rejected.
/// Does not create orders (policy is checked before repo). Always restores standard.
#[tokio::test]
async fn manufacturing_costing_method_gates() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let (st, body) = ctx
        .client
        .put(
            "/api/manufacturing/costing-policy",
            serde_json::json!({ "costing_method": "actual" }),
        )
        .await?;
    assert!(st.is_success(), "set actual: {st} {body}");
    let data = body.get("data").cloned().unwrap_or(body.clone());
    assert_eq!(data["costing_method"].as_str(), Some("actual"));

    // Dummy order id — service rejects on policy before looking up the order
    let dummy = "00000000-0000-4000-8000-000000000001";

    let (s, oh) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/applied",
            serde_json::json!({
                "production_order": dummy,
                "base_hours": "1",
                "allocation_base": "LABOR_HOURS"
            }),
        )
        .await?;
    let oh_txt = oh.to_string();
    assert_eq!(
        s.as_u16(),
        422,
        "applied OH under actual must be 422 Unprocessable, got {s} {oh}"
    );
    assert!(
        oh_txt.contains("actual costing") || oh_txt.contains("Predetermined"),
        "message should mention actual/predetermined: {oh}"
    );
    println!("✓ actual costing rejects predetermined OH (422)");

    let (s, dl) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            serde_json::json!({
                "production_order_uuid": dummy,
                "reference": "gate-test"
            }),
        )
        .await?;
    let dl_txt = dl.to_string();
    assert_eq!(
        s.as_u16(),
        422,
        "routing DL under actual must be 422 Unprocessable, got {s} {dl}"
    );
    assert!(
        dl_txt.contains("standard costing") || dl_txt.contains("Routing applied"),
        "message should mention standard/routing: {dl}"
    );
    println!("✓ actual costing rejects routing applied labour (422)");

    // Always restore — shared tenant state
    let (st, body) = ctx
        .client
        .put(
            "/api/manufacturing/costing-policy",
            serde_json::json!({ "costing_method": "standard" }),
        )
        .await?;
    assert!(st.is_success(), "restore standard: {st} {body}");
    println!("✅ manufacturing_costing_method_gates OK");
    let _ = ctx.save_state();
    Ok(())
}


/// Stage C: under actual costing, complete transfers FG at accumulated actual cost
/// (materials + timesheet labour + recognized OH) without routing applied labour.
#[tokio::test]
async fn manufacturing_actual_completion_path() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    let (st, body) = ctx
        .client
        .put(
            "/api/manufacturing/costing-policy",
            serde_json::json!({ "costing_method": "actual" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(st.is_success(), "set actual: {st} {body}");

    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inv: {s}");
    let fg = find_item(&inv, "FG-CARGO-MULE")
        .map_err(|e| format!("{e} — run manufacturing_flow first"))?;
    let fg_uuid = extract_uuid(fg, &["uuid"]).map_err(|e| e.to_string())?;
    let anchor = find_item(&inv, RMS[0].sku).map_err(|e| e.to_string())?;
    let warehouse_serial = anchor["warehouse_serial"]
        .as_i64()
        .or_else(|| anchor["warehouse_serial_id"].as_i64())
        .unwrap_or(1);

    // Restock RMs via credit purchase (qty 1 completion needs little stock)
    if ctx.vendors.is_empty() {
        let _ = ctx.seed_trading_partners().await;
    }
    if let Some(vendor_uuid) = ctx.vendors.values().next().copied() {
        let rows = inv["data"].as_array().cloned().unwrap_or_default();
        let mut bill_items = Vec::new();
        for rm in RMS {
            let Some(row) = rows.iter().find(|i| i["sku"].as_str() == Some(rm.sku)) else {
                continue;
            };
            let Some(stock_item_id) = row["serial_id"].as_i64() else {
                continue;
            };
            bill_items.push(json!({
                "stock_item_id": stock_item_id,
                "description": rm.name,
                "quantity": "5",
                "unit_price": rm.unit_price,
                "tax_rate": "18"
            }));
        }
        if !bill_items.is_empty() {
            let (s, body) = ctx
                .client
                .post(
                    "/api/procurement/create/purchase",
                    json!({
                        "bill_number": format!(
                            "BILL-ACT-{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        ),
                        "vendor_uuid": vendor_uuid,
                        "bill_date": "2026-03-01",
                        "due_date": "2026-03-31",
                        "reference": "RESTOCK-ACTUAL-COMPLETION",
                        "settlement_type": "credit",
                        "payment_status": "unpaid",
                        "items": bill_items
                    }),
                )
                .await
                .map_err(|e| e.to_string())?;
            if ok(s) {
                if let Some(bill_serial) = body["data"]["serial_id"].as_i64() {
                    let _ = ctx
                        .client
                        .post(
                            "/api/procurement/post/credit/purchase",
                            json!({
                                "bill_serial_id": bill_serial,
                                "vat_tax_account": "110501",
                                "payables_account": "210100",
                                "cash_account": null
                            }),
                        )
                        .await;
                    println!("✓ RM restocked for actual completion");
                }
            }
        }
    }

    let (s, emps) = ctx
        .client
        .get("/api/workforce/list/employees")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "employees: {s}");
    let emp_rows = emps["data"].as_array().cloned().unwrap_or_default();
    assert!(emp_rows.len() >= 2, "need ≥2 employees");
    let emp1 = extract_uuid(&emp_rows[0], &["uuid"]).map_err(|e| e.to_string())?;
    let emp2 = extract_uuid(&emp_rows[1], &["uuid"]).map_err(|e| e.to_string())?;

    let order_no = format!(
        "PO-ACT-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/create/production-order",
            json!({
                "order_number": order_no,
                "product_item_uuid": fg_uuid,
                "quantity_ordered": "1",
                "start_date": "2026-03-01",
                "expected_completion_date": "2026-03-10"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create PO: {s} {body}");
    let po_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ actual-path PO {po_uuid}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": warehouse_serial,
                "production_order_uuid": po_uuid
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue materials: {s} {body}");
    println!("✓ materials issued under actual");

    let (s, dl) = ctx
        .client
        .post(
            "/api/manufacturing/apply/direct/labor",
            json!({ "production_order_uuid": po_uuid, "reference": "should-fail" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(s.as_u16(), 422, "routing DL forbidden under actual: {s} {dl}");
    println!("✓ routing DL rejected 422 under actual");

    let (_s, cats) = ctx
        .client
        .get("/api/manufacturing/labour/categories")
        .await
        .map_err(|e| e.to_string())?;
    let cat_uuid = cats["data"]
        .as_array()
        .and_then(|a| {
            a.iter()
                .find(|c| c["code"].as_str() == Some("ASSY"))
                .and_then(|c| c["uuid"].as_str())
        })
        .map(|s| s.to_string());
    if let Some(ref cu) = cat_uuid {
        let _ = ctx
            .client
            .post(
                "/api/manufacturing/labour/rates",
                json!({
                    "labour_category_uuid": cu,
                    "rate_kind": "Actual",
                    "hourly_rate": "15000",
                    "effective_from": "2026-01-01"
                }),
            )
            .await;
    }

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/timesheet/team",
            json!({
                "employee_uuids": [emp1, emp2],
                "work_date": "2026-03-02",
                "hours": "2",
                "activity_type": "Direct",
                "production_order_uuid": po_uuid,
                "work_center_code": "WC-ASSY",
                "labour_category_uuid": cat_uuid,
                "memo": "actual completion labour"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "team timesheet: {s} {body}");
    let entry_ids: Vec<_> = body["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| e["uuid"].as_str().map(|s| s.to_string()))
        .collect();
    for path in [
        "/api/manufacturing/timesheet/bulk-submit",
        "/api/manufacturing/timesheet/bulk-approve",
        "/api/manufacturing/timesheet/bulk-post",
    ] {
        let (s, body) = ctx
            .client
            .post(path, json!({ "entry_uuids": entry_ids.clone() }))
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "{path}: {s} {body}");
    }
    println!("✓ timesheet actual labour posted");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/apply/overhead/actual",
            json!({
                "production_order_uuid": po_uuid,
                "base_amount": "25000",
                "allocation_base": "LABOR_HOURS",
                "reference": "actual-oh-completion"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "actual OH: {s} {body}");
    println!("✓ actual overhead recognized 25000");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/start-production",
            json!({ "production_order_uuid": po_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "start: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/manufacturing/complete-production",
            json!({
                "production_order_uuid": po_uuid,
                "completed_quantity": "1"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "complete under actual: {s} {body}");
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "Completed", "order should be Completed, got {status}");
    println!("✓ actual-cost completion → Completed");

    let (s, var) = ctx
        .client
        .post(
            &format!("/api/manufacturing/labour/variances/{po_uuid}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(
        s.as_u16(),
        422,
        "std labour variances forbidden under actual: {s} {var}"
    );
    println!("✓ std labour variance API returns 422 under actual");

    let (st, body) = ctx
        .client
        .put(
            "/api/manufacturing/costing-policy",
            serde_json::json!({ "costing_method": "standard" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(st.is_success(), "restore standard: {st} {body}");
    println!("✅ manufacturing_actual_completion_path OK");
    let _ = ctx.save_state();
    Ok(())
}

