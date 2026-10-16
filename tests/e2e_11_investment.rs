mod common;

use common::client::extract_uuid;
use common::{E2eResult, TestContext};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn investment_case_pipeline() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Currency from COA currencies endpoint (CreateInvestmentCase requires currency_id)
    /*let (s, cur) = ctx
        .client
        .get("/api/accounts/currencies")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "currencies: {s} {cur}");
    let currency_id = cur["data"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|c| {
            extract_uuid(c, &["uuid"])
                .or_else(|_| extract_uuid(c, &["id"]))
                .ok()
        })
        .ok_or("need a currency uuid")?;
	*/
	let currency_id = ctx
	    .currency_id("UGX")
    	.ok_or("UGX missing — run e2e_02 seed_currencies")?;

    // ----- Program -----
    let (s, body) = ctx
        .client
        .post(
            "/api/investment/programs",
            json!({
                "program_code": format!("PROG-{}", chrono::Utc::now().timestamp_millis() % 100000),
                "name": "Capacity Expansion 2025",
                "description": "E2E program",
                "horizon_start": "2025-01-01",
                "horizon_end": "2027-12-31"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create program: {s} {body}");
    let program_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ program {program_id}");

    // ----- Case -----
    let (s, body) = ctx
        .client
        .post(
            "/api/investment/cases",
            json!({
                "case_code": format!("CASE-{}", chrono::Utc::now().timestamp_millis() % 100000),
                "title": "New packaging line",
                "description": "E2E expansion case",
                "investment_type": "EXPANSION",
                "program_id": program_id,
                "strategic_score": "8.5",
                "risk_rating": "MEDIUM",
                "currency_id": currency_id,
                "hurdle_rate": "0.12"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create case: {s} {body}");
    let case_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ case {case_id}");

    // ----- Scenario -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/investment/cases/{case_id}/scenarios"),
            json!({
                "scenario_code": "BASE",
                "name": "Base case",
                "probability": "0.6",
                "inflation_rate": "0.05",
                "is_primary": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create scenario: {s} {body}");
    let scenario_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ scenario {scenario_id}");

    // ----- Cash flows (t0 CAPEX + inflows) -----
    for (period, line_type, amount, desc) in [
        (0, "CAPEX", "-50000000", "Equipment outlay"),
        (1, "REVENUE", "18000000", "Year 1 revenue"),
        (2, "REVENUE", "22000000", "Year 2 revenue"),
        (3, "REVENUE", "25000000", "Year 3 revenue"),
    ] {
        let (s, body) = ctx
            .client
            .post(
                &format!("/api/investment/scenarios/{scenario_id}/cashflows"),
                json!({
                    "period_no": period,
                    "period_date": format!("202{}-12-31", 5 + period),
                    "line_type": line_type,
                    "description": desc,
                    "amount": amount,
                    "is_cash": true,
                    "tax_deductible": line_type == "CAPEX"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "cashflow {period}: {s} {body}");
    }
    println!("✓ cashflows added");

    // ----- Recalculate metrics -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/investment/cases/{case_id}/scenarios/{scenario_id}/recalculate"),
            json!({ "discount_rate": "0.12" }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "recalculate: {s} {body}");
    println!("✓ metrics recalculated");

    // ----- Portfolio board -----
    let (s, body) = ctx
        .client
        .get("/api/investment/portfolio")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "portfolio: {s} {body}");
    println!("✓ portfolio board");

    // Investment is decisioning, not always GL — still assert TB unbroken
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ investment_case_pipeline OK for {}", ctx.company_name);
    Ok(())
}
