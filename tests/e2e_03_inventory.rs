//! e2e_04b_inventory — warehouses, categories, RM / SA / FG items.
//! Procurement buys; manufacturing builds; sales sells. No creates elsewhere.

mod common;

use common::client::extract_uuid;
use common::fixtures::{
    RmSpec, FG_CARGO_MULE, RMS, SA_CTRL_CORE, SA_PROP_GROUND,
};
use common::{E2eResult, TestContext};
use serde_json::{json, Value};

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

fn acct(ctx: &TestContext, codes: &[&str]) -> String {
    for c in codes {
        if ctx.account_uuid(c).is_some() {
            return (*c).to_string();
        }
    }
    codes[0].to_string()
}

fn find_by_sku<'a>(inv: &'a Value, sku: &str) -> Result<&'a Value, String> {
    inv["data"]
        .as_array()
        .ok_or_else(|| String::from("inventory data not an array"))?
        .iter()
        .find(|i| i["sku"].as_str() == Some(sku))
        .ok_or_else(|| format!("sku {sku} missing"))
}

#[tokio::test]
async fn inventory_master_data() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let inv_rm = acct(&ctx, &["110701", "110700"]);
    let inv_fg = acct(&ctx, &["110703", "110701"]);
    let cogs = acct(&ctx, &["510100", "500000"]);
    let income = acct(&ctx, &["410000", "410100"]);

    // =========================================================================
    // 1. Warehouses
    // =========================================================================
    let warehouses = [
        ("WH-RM", "Raw Materials Store", "Electronics & mechanical leaves"),
        ("WH-FG", "Finished Goods Store", "Ship-ready robots"),
        ("WH-WIP", "WIP Floor", "Optional bin — not a stock item"),
    ];
    let mut wh_serials = std::collections::HashMap::new();
    let mut wh_uuids = std::collections::HashMap::new();

    for (code, name, desc) in warehouses {
        let (s, body) = ctx
            .client
            .post(
                "/api/inventory/create/warehouse",
                json!({
                    "code": code,
                    "name": name,
                    "description": desc,
                    "location": "Kampala"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "warehouse {code}: {s} {body}");
        let serial = body["data"]["serial_id"].as_i64().ok_or("wh serial")?;
        let uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
        wh_serials.insert(code, serial);
        wh_uuids.insert(code, uuid.clone());
        println!("✓ warehouse {code} serial={serial}");
    }

    let wh_rm = *wh_serials.get("WH-RM").unwrap();
    let wh_fg = *wh_serials.get("WH-FG").unwrap();

    // list + get warehouse
    let (s, body) = ctx
        .client
        .get("/api/inventory/list/warehouses")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list warehouses: {s} {body}");
    let n_wh = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n_wh >= 3, "expected ≥3 warehouses, got {n_wh}");

    let wh_uuid = wh_uuids.get("WH-RM").unwrap();
    let (s, body) = ctx
        .client
        .get(&format!("/api/inventory/get/warehouse/{wh_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get warehouse: {s} {body}");
    println!("✓ warehouses list/get");

    // patch warehouse
    let (s, body) = ctx
        .client
        .patch(
            &format!("/api/inventory/update/warehouse/{wh_uuid}"),
            json!({
                "code": "WH-RM",
                "name": "Raw Materials Store",
                "description": "Electronics & mechanical leaves (updated)",
                "location": "Kampala"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "update warehouse: {s} {body}");
    println!("✓ warehouse updated");

    // =========================================================================
    // 2. Categories
    // =========================================================================
    let categories = [
        ("RAW-ELEC", "Raw – Electronics"),
        ("RAW-MECH", "Raw – Mechanical"),
        ("SA", "Subassemblies"),
        ("FG", "Finished goods"),
    ];
    let mut cat_uuids = std::collections::HashMap::new();

    for (code, name) in categories {
        let (s, body) = ctx
            .client
            .post(
                "/api/inventory/create/category",
                json!({
                    "code": code,
                    "name": name,
                    "description": name
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "category {code}: {s} {body}");
        let uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
        cat_uuids.insert(code, uuid.clone());
        println!("✓ category {code}");
    }

    let (s, body) = ctx
        .client
        .get("/api/inventory/list/categories")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list categories: {s} {body}");
    assert!(
        body["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 4,
        "expected ≥4 categories"
    );

    let cat_raw = cat_uuids.get("RAW-ELEC").unwrap().clone();
    let (s, body) = ctx
        .client
        .get(&format!("/api/inventory/get/category/{cat_raw}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get category: {s} {body}");
    println!("✓ categories list/get");

    // =========================================================================
    // 3. RM items (purchased later in e2e_05)
    // =========================================================================
    for rm in RMS {
        let cat = if rm.sku.starts_with("FRM") || rm.sku.starts_with("FST") {
            cat_uuids.get("RAW-MECH").unwrap()
        } else {
            cat_uuids.get("RAW-ELEC").unwrap()
        };
        let (s, body) = ctx
            .client
            .post(
                "/api/inventory/create/item",
                json!({
                    "sku": rm.sku,
                    "name": rm.name,
                    "warehouse_serial": wh_rm,
                    "category_uuid": cat,
                    "unit": rm.unit,
                    "selling_price": "0",
                    "standard_cost": rm.standard_cost,
                    "track_quantity": true,
                    "reorder_level": "5",
                    "asset_account": inv_rm,
                    "cogs_account": cogs,
                    "income_account": income
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "RM {}: {s} {body}", rm.sku);
        println!("✓ RM {}", rm.sku);
    }

    // =========================================================================
    // 4. Subassembly + FG shells (qty 0 until manufacturing / sales)
    // =========================================================================
    let sa_cat = cat_uuids.get("SA").unwrap();
    let fg_cat = cat_uuids.get("FG").unwrap();

    for (sku, name, cost, price, wh) in [
        (SA_CTRL_CORE, "Control core subassembly", "150000", "0", wh_rm),
        (SA_PROP_GROUND, "Ground propulsion subassembly", "200000", "0", wh_rm),
        (FG_CARGO_MULE, "CargoMule site delivery robot", "2500000", "4500000", wh_fg),
    ] {
        let cat = if sku.starts_with("FG") { fg_cat } else { sa_cat };
        let (s, body) = ctx
            .client
            .post(
                "/api/inventory/create/item",
                json!({
                    "sku": sku,
                    "name": name,
                    "warehouse_serial": wh,
                    "category_uuid": cat,
                    "unit": "pc",
                    "selling_price": price,
                    "standard_cost": cost,
                    "track_quantity": true,
                    "reorder_level": "1",
                    "asset_account": if sku.starts_with("FG") { &inv_fg } else { &inv_rm },
                    "cogs_account": cogs,
                    "income_account": income
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "item {sku}: {s} {body}");
        println!("✓ item {sku}");
    }

    // =========================================================================
    // 5. List / get / patch item (catalog stays; temp row for delete)
    // =========================================================================
    let (s, inv) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inventory: {s} {inv}");
    let n = inv["data"].as_array().map(|a| a.len()).unwrap_or(0);
    // RMS + 2 SA + 1 FG
    assert!(
        n >= RMS.len() + 3,
        "expected ≥{} items, got {n}",
        RMS.len() + 3
    );

    for rm in RMS {
        let _ = find_by_sku(&inv, rm.sku)?;
    }
    let _ = find_by_sku(&inv, FG_CARGO_MULE)?;
    let _ = find_by_sku(&inv, SA_CTRL_CORE)?;
    let _ = find_by_sku(&inv, SA_PROP_GROUND)?;
    println!("✓ list inventory n={n} (all catalog SKUs present)");

    let fg_row = find_by_sku(&inv, FG_CARGO_MULE)?;
    let fg_uuid = extract_uuid(fg_row, &["uuid"]).map_err(|e| e.to_string())?;
    let (s, body) = ctx
        .client
        .get(&format!("/api/inventory/get/item/{fg_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get FG item: {s} {body}");
    println!("✓ get item FG-CARGO-MULE");

    // patch FG reorder only (full CreateItem body required by API)
    let (s, body) = ctx
        .client
        .patch(
            &format!("/api/inventory/update/item/{fg_uuid}"),
            json!({
                "sku": FG_CARGO_MULE,
                "name": "CargoMule site delivery robot",
                "warehouse_serial": wh_fg,
                "category_uuid": fg_cat,
                "unit": "pc",
                "selling_price": "4500000",
                "standard_cost": "2500000",
                "track_quantity": true,
                "reorder_level": "2",
                "asset_account": inv_fg,
                "cogs_account": cogs,
                "income_account": income
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "update FG: {s} {body}");
    println!("✓ update item FG");

    // Create → delete disposable item (exercise delete without harming catalog)
    let (s, body) = ctx
        .client
        .post(
            "/api/inventory/create/item",
            json!({
                "sku": "TMP-DELETE-ME",
                "name": "Temporary scrap SKU",
                "warehouse_serial": wh_rm,
                "category_uuid": cat_raw,
                "unit": "pc",
                "selling_price": "0",
                "standard_cost": "1",
                "track_quantity": true,
                "reorder_level": "0",
                "asset_account": inv_rm,
                "cogs_account": cogs,
                "income_account": income
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "temp item: {s} {body}");
    let tmp_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let (s, body) = ctx
        .client
        .delete(&format!("/api/inventory/delete/item/{tmp_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "delete temp item: {s} {body}");
    println!("✓ delete disposable item");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ inventory_master_data OK for {}", ctx.company_name);
    Ok(())
}
