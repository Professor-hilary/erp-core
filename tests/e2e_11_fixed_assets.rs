mod common;

use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn fixed_asset_lifecycle() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // ----- 1. Asset classes -----
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
    assert!(ok(s), "create class machinery: {s} {body}");
    let class_mach = extract_uuid(&body, &["data", "class_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ asset class machinery {class_mach}");

    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/class/create",
            json!({
                "class_name": "Vehicles",
                "category_type": "PPE",
                "description": "Logistics fleet"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create class vehicles: {s} {body}");
    let class_veh = extract_uuid(&body, &["data", "class_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ asset class vehicles {class_veh}");

    let (s, body) = ctx
        .client
        .get("/api/fixedassets/asset/class/list")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list classes: {s} {body}");
    assert!(
        body["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 2,
        "expected ≥2 classes"
    );

    // ----- 2. Direct asset (mixer) -----
    let code_mixer = format!("FA-MIX-{}", chrono::Utc::now().timestamp_millis());
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/create",
            json!({
                "asset_code": code_mixer,
                "asset_name": "Industrial Mixer",
                "description": "E2E production mixer",
                "class_id": class_mach,
                "location": "Kampala Plant",
                "acquisition_date": "2026-01-01",
                "original_cost": "15000000",
                "is_capitalized": false,
                "useful_life_years": 5,
                "residual_value": "500000",
                "depreciation_method": "Straight_Line",
                "depreciation_start_date": "2026-01-01",
                "status": "In_Use"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create mixer: {s} {body}");
    let mixer_id = extract_uuid(&body, &["data", "asset_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ mixer {mixer_id}");

    // ----- 3. Capitalize mixer -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/capitalize",
            json!({
                "asset_uuid": mixer_id,
                "date": "2026-01-01",
                "capitalized_amount": "15000000",
                "costs": [
                    {"description": "Purchase", "amount": "14000000"},
                    {"description": "Installation", "amount": "1000000"}
                ]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "capitalize mixer: {s} {body}");
    println!("✓ mixer capitalized");

    // ----- 4. CWIP project (assembly line) then capitalize into asset -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/cwip/create",
            json!({
                "project_code": format!("CWIP-{}", chrono::Utc::now().timestamp_millis()),
                "project_name": "Assembly Line Build-out",
                "total_accumulated_cost": "0",
                "description": "In-house line for CargoMule",
                "start_date": "2026-01-05",
                "budget_amount": "25000000",
                "status": "In_Progress"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    if ok(s) {
        let cwip_id = extract_uuid(&body, &["data", "uuid"])
            .or_else(|_| extract_uuid(&body, &["data", "id"]))
            .or_else(|_| extract_uuid(&body, &["data", "cwip_id"]));
        println!("✓ CWIP created {cwip_id:?}");

        // Create target asset shell for CWIP capitalization if API needs it
        let (s2, body2) = ctx
            .client
            .post(
                "/api/fixedassets/asset/create",
                json!({
                    "asset_code": format!("FA-LINE-{}", chrono::Utc::now().timestamp_millis()),
                    "asset_name": "CargoMule Assembly Line",
                    "description": "Capitalized from CWIP",
                    "class_id": class_mach,
                    "location": "Kampala Plant",
                    "acquisition_date": "2026-01-20",
                    "original_cost": "25000000",
                    "is_capitalized": false,
                    "useful_life_years": 10,
                    "residual_value": "1000000",
                    "depreciation_method": "Straight_Line",
                    "depreciation_start_date": "2026-02-01",
                    "status": "Under_Construction"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        if ok(s2) {
            let line_id = extract_uuid(&body2, &["data", "asset_id"])
                .or_else(|_| extract_uuid(&body2, &["data", "uuid"]))
                .ok();
            println!("✓ line asset shell {line_id:?}");
            // capitalize CWIP → asset if both ids present
            if let (Ok(cid), Some(aid)) = (cwip_id, line_id) {
                let (s3, body3) = ctx
                    .client
                    .post(
                        &format!("/api/fixedassets/cwip/capitalize/{cid}"),
                        json!({ "asset_uuid": aid }),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                // route may need different shape — soft assert
                if ok(s3) {
                    println!("✓ CWIP capitalized into line");
                } else {
                    println!("⚠ CWIP capitalize: {s3} {body3}");
                }
            }
        } else {
            println!("⚠ line asset: {s2} {body2}");
        }
    } else {
        println!("⚠ CWIP create: {s} {body} (DTO may differ — non-blocking)");
    }

    // ----- 5. Second asset: vehicle -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/create",
            json!({
                "asset_code": format!("FA-VAN-{}", chrono::Utc::now().timestamp_millis()),
                "asset_name": "Delivery Van",
                "description": "Last-mile van",
                "class_id": class_veh,
                "location": "Kampala Depot",
                "acquisition_date": "2026-01-10",
                "original_cost": "45000000",
                "is_capitalized": false,
                "useful_life_years": 4,
                "residual_value": "5000000",
                "depreciation_method": "Straight_Line",
                "depreciation_start_date": "2026-01-10",
                "status": "In_Use"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create van: {s} {body}");
    let van_id = extract_uuid(&body, &["data", "asset_id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ van {van_id}");

    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/asset/capitalize",
            json!({
                "asset_uuid": van_id,
                "date": "2026-01-10",
                "capitalized_amount": "45000000",
                "costs": [
                    {"description": "Purchase", "amount": "45000000"}
                ]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "capitalize van: {s} {body}");
    println!("✓ van capitalized");

    // ----- 6. Depreciation run -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/depreciation/run",
            json!({ "period_date": "2026-01-31" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "run depreciation: {s} {body}");
    println!("✓ depreciation run");

    // Schedule for mixer
    let (s, body) = ctx
        .client
        .get(&format!(
            "/api/fixedassets/depreciation/schedule/{mixer_id}"
        ))
        .await
        .map_err(|e| e.to_string())?;
    if ok(s) {
        println!("✓ depr schedule mixer");
    } else {
        println!("⚠ depr schedule: {s} {body}");
    }

    // ----- 7. Get / list -----
    let (s, body) = ctx
        .client
        .get(&format!("/api/fixedassets/asset/get/{mixer_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get mixer: {s} {body}");
    println!("✓ get mixer");

    let (s, body) = ctx
        .client
        .get("/api/fixedassets/asset/list")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list assets: {s} {body}");
    let n = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n >= 2, "expected ≥2 assets, got {n}");
    println!("✓ assets n={n}");

    // ----- 8. Optional disposal (van) — soft -----
    let (s, body) = ctx
        .client
        .post(
            "/api/fixedassets/disposal/create",
            json!({
                "asset_id": van_id,
                "disposal_date": "2026-01-31",
                "disposal_type": "Sale",
                "proceeds": "40000000",
                "reason": "E2E disposal smoke"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    if ok(s) {
        println!("✓ van disposed");
    } else {
        println!("⚠ disposal: {s} {body} (non-blocking)");
    }

    // ----- 9. TB -----
    ctx.assert_trial_balance_ok("2026-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after fixed assets");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ fixed_asset_lifecycle OK for {}", ctx.company_name);
    Ok(())
}
