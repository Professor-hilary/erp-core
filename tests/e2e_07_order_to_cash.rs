mod common;

use common::client::extract_uuid;
use common::{E2eResult, TestContext};
use serde_json::json;
use common::fixtures::{RmSpec, RMS, FG_CARGO_MULE};

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn order_to_cash() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    if ctx.customers.is_empty() {
        ctx.seed_trading_partners()
            .await
            .map_err(|e| e.to_string())?;
    }
    assert!(
        !ctx.customers.is_empty(),
        "need at least one customer — run e2e_02 first"
    );
    let customer_uuid = *ctx.customers.values().next().expect("customer");

    // ----- Ensure we have a sellable stock item (reuse from P2P or create) -----
    let (s, items_body) = ctx
        .client
        .get("/api/inventory/list/inventory")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list inventory: {s} {items_body}");

    //let stock_item_id = items_body["data"]
    //    .as_array()
    //    .and_then(|arr| arr.first())
    //    .and_then(|i| i["serial_id"].as_i64())
    //    .unwrap_or(0);

	let stock_item_id = items_body["data"].as_array().unwrap().iter()
	    .find(|i| i["sku"].as_str() == Some(FG_CARGO_MULE))
    	.and_then(|i| i["serial_id"].as_i64())
    	.ok_or("FG-CARGO-MULE missing — run e2e_06")?;

    let stock_item_id = if stock_item_id > 0 {
        println!("✓ reuse item serial={stock_item_id}");
        stock_item_id
    } else {
        // Minimal warehouse + item (same as e2e_06)
        let (s, body) = ctx
            .client
            .post(
                "/api/inventory/create/warehouse",
                json!({
                    "code": "WH-SALES",
                    "name": "Sales Staging",
                    "description": "For O2C",
                    "location": "Kampala"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "warehouse: {s} {body}");
        let wh = body["data"]["serial_id"].as_i64().ok_or("wh serial")?;

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
                    "sku": "FG-BREAD-1",
                    "name": "White Bread Loaf",
                    "warehouse_serial": wh,
                    "description": "Finished goods",
                    "unit": "loaf",
                    "selling_price": "5000",
                    "standard_cost": "2500",
                    "track_quantity": true,
                    "reorder_level": "20",
                    "asset_account": inv_code,
                    "cogs_account": "510100",
                    "income_account": "410000"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "create item: {s} {body}");
        let id = body["data"]["serial_id"].as_i64().ok_or("item serial")?;
        println!("✓ created item serial={id}");
        id
    };

    // ----- Create credit invoice (unposted) -----
    let invoice_number = format!("INV-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/sales/create/sale",
            json!({
                "invoice_number": invoice_number,
                "customer_uuid": customer_uuid,
                "issue_date": "2025-01-25",
                "due_date": "2025-02-25",
                "settlement_type": "credit",
                "items": [{
                    "stock_item_id": stock_item_id,
                    "description": "White Bread Loaf",
                    "quantity": "20",
                    "unit_price": "5000",
                    "tax_rate": "18"
                }]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create sale: {s} {body}");
    let inv_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    println!("✓ invoice {invoice_number} uuid={inv_uuid}");

    // ----- Post credit invoice → AR + revenue + output VAT (+ stock/COGS via DB fn) -----
    // Adjust codes if your COA differs.
    let (s, body) = ctx
        .client
        .post(
            "/api/sales/post/sale/invoice",
            json!({
                "invoice_number": invoice_number,
                "receivable_code": "110300",   // Accounts Receivable (adjust if needed)
                "revenue_code": "400000",
                "output_vat_code": "210300",   // Output VAT liability (adjust if needed)
                "cash_account_code": null
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post invoice: {s} {body}");
    println!("✓ credit invoice posted");

    // ----- List customer invoices -----
    let (s, body) = ctx
        .client
        .get(&format!("/api/sales/list/invoices/{customer_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list invoices: {s} {body}");
    let n = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n >= 1, "expected at least one invoice");
    println!("✓ customer invoices n={n}");

    // ----- Trial balance -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after order-to-cash");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ order_to_cash OK for {}", ctx.company_name);
    Ok(())
}
