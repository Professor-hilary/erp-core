mod common;

use common::client::extract_uuid;
use common::{E2eResult, TestContext};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn fixed_asset_lifecycle() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // ----- 1. Asset class -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/class/create",
            json!({
                "class_name": "Plant & Machinery",
                "category_type": "PPE",
                "description": "Production equipment"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create class: {s} {body}");
    // handlers return class_id on AssetClass
    let class_id = extract_uuid(&body, &["data", "class_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ asset class {class_id}");

    // ----- 2. Create asset (not yet capitalized) -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/create",
            json!({
                "asset_code": format!("FA-{}", chrono::Utc::now().timestamp_millis()),
                "asset_name": "Industrial Mixer",
                "description": "E2E production mixer",
                "class_id": class_id,
                "location": "Kampala Plant",
                "acquisition_date": "2025-01-01",
                "original_cost": "15000000",
                "is_capitalized": false,
                "useful_life_years": 5,
                "residual_value": "500000",
                "depreciation_method": "Straight_Line",
                "depreciation_start_date": "2025-01-01",
                "status": "In_Use"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create asset: {s} {body}");
    let asset_id = extract_uuid(&body, &["data", "asset_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ asset {asset_id}");

    // ----- 3. Capitalize -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/capitalize",
            json!({
                "asset_uuid": asset_id,
                "date": "2025-01-01",
                "capitalized_amount": "15000000",
                "costs": [
                    {"description": "Purchase", "amount": "14000000"},
                    {"description": "Installation", "amount": "1000000"}
                ]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "capitalize: {s} {body}");
    println!("✓ capitalized");

    // ----- 4. Run depreciation for period -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/depreciation/run",
            json!({ "period_date": "2025-01-31" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "run depreciation: {s} {body}");
    println!("✓ depreciation run");

    // ----- 5. List assets -----
    let (s, body) = ctx
        .client
        .get("/api/fixedassets/asset/list")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list assets: {s} {body}");
    let n = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n >= 1, "expected ≥1 asset");
    println!("✓ assets n={n}");

    // ----- 6. TB -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after fixed assets");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ fixed_asset_lifecycle OK for {}", ctx.company_name);
    Ok(())
}
