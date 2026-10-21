mod common;

use common::client::extract_uuid;
use common::{E2eResult, TestContext};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

fn acct_uuid(ctx: &TestContext, codes: &[&str]) -> Result<uuid::Uuid, String> {
    for c in codes {
        if let Some(u) = ctx.account_uuid(c) {
            return Ok(u);
        }
    }
    Err(format!("none of {codes:?} present in COA"))
}

/// Existing equity path (crude journal) — keep so later modules still see cash/equity.
#[tokio::test]
async fn raise_equity() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.raise_capital("20000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;

    let txn_id = ctx
        .raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .get(&format!("/api/transactions/get/{txn_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(s.is_success(), "get journal: {body}");

    let (s, _list) = ctx.client.get("/api/transactions/list/brief").await?;
    assert!(s.is_success());
    Ok(())
}

/// Debt facility lifecycle: facility → lender → drawdown → accrual → schedule → repayment.
#[tokio::test]
async fn debt_facility_lifecycle() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let ugx = ctx
        .currency_id("UGX")
        .ok_or_else(|| "UGX not seeded — run e2e_02 first".to_string())?;

    let cash = acct_uuid(&ctx, &["110200", "110100"])?;
    let loan_liab = acct_uuid(&ctx, &["220100", "220200"])?;
    let int_exp = acct_uuid(&ctx, &["640100"])?;
    let int_accrued = acct_uuid(&ctx, &["210400", "210500"])?;

    // ----- 1. Pick a lender party (seeded in e2e_02) -----
    let (s, parties) = ctx
        .client
        .get("/api/capital/party")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list parties: {s} {parties}");
    let lender = parties["data"]
        .as_array()
        .ok_or("parties data not array")?
        .iter()
        .find(|p| {
            p["party_type"].as_str() == Some("LENDER")
                || p["role"].as_str() == Some("lender_contact")
        })
        .ok_or("no LENDER party — run e2e_02")?;
    let lender_id = extract_uuid(lender, &["id"])
        .or_else(|_| extract_uuid(lender, &["uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ lender party {lender_id}");

    // ----- 2. Create term facility (100m UGX, 3 years) -----
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/facilities",
            json!({
                "facility_code": "FAC-TERM-001",
                "facility_name": "DFCU CapEx Term Loan",
                "facility_type": "TERM_LOAN",
                "agent_id": lender_id,
                "currency_id": ugx,
                "committed_amount": "100000000",
                "interest_type": "FIXED",
                "base_rate_index": null,
                "margin_bps": "1200",              // 12%
                "day_count_convention": "ACT/365",
                "payment_frequency": "QUARTERLY",
                "amortization_type": "AMORTIZING",
                "effective_date": "2025-01-10",
                "maturity_date": "2028-01-10",
                "commitment_fee_bps": "50",
                "collateral_required": true,
                "is_secured": true,
                "ranking": "SENIOR",
                "status": "active"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create facility: {s} {body}");
    let facility_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ facility {facility_id}");

    // ----- 3. Add lender commitment -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/lenders"),
            json!({
                "lender_id": lender_id,
                "commitment_amount": "100000000",
                "participation_pct": "100",
                "is_agent": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "add lender: {s} {body}");
    println!("✓ facility lender");

    // ----- 4. Drawdown 60m (Dr Bank / Cr LT Loan) -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/drawdowns"),
            json!({
                "amount": "60000000",
                "drawdown_date": "2025-01-15",
                "value_date": "2025-01-15",
                "currency_id": ugx,
                "cash_account_code": cash,
                "loan_liability_code": loan_liab,
                "reference": "DD-001",
                "purpose": "PPE and production tooling"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "drawdown: {s} {body}");
    let drawdown_id = extract_uuid(&body, &["data", "drawdown_id"])
        .or_else(|_| extract_uuid(&body, &["data", "id"]))
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ drawdown {drawdown_id}");

    // List drawdowns
    let (s, body) = ctx
        .client
        .get(&format!("/api/capital/facilities/{facility_id}/drawdowns"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list drawdowns: {s} {body}");
    let n_dd = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n_dd >= 1, "expected ≥1 drawdown");
    println!("✓ drawdowns n={n_dd}");

    // ----- 5. Interest accrual (Jan 15–31 on 60m @ 12% ACT/365) -----
    // 60_000_000 * 0.12 * 16/365 ≈ 315_616
    let interest_amt = "315616";
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/accruals"),
            json!({
                "period_start": "2025-01-15",
                "period_end": "2025-01-31",
                "principal_base": "60000000",
                "annual_rate": "0.12",
                "day_count": 16,
                "interest_amount": interest_amt,
                "is_paid": false,
                "interest_exp_id": int_exp,
                "interest_acc_id": int_accrued
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "accrual: {s} {body}");
    println!("✓ interest accrual {interest_amt}");

    // ----- 6. Repayment schedule (first quarterly installment) -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/schedules"),
            json!({
                "sequence_no": 1,
                "due_date": "2025-04-15",
                "principal_due": "5000000",
                "interest_due": interest_amt,
                "fee_due": "0",
                "status": "SCHEDULED"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "schedule: {s} {body}");
    let schedule_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .ok(); // optional link on repayment
    println!("✓ schedule seq=1");

    // ----- 7. Partial repayment (pay the accrued interest + 5m principal early for test) -----
    let mut repay_body = json!({
        "repayment_date": "2025-01-31",
        "principal_amount": "5000000",
        "interest_amount": interest_amt,
        "fee_amount": "0",
        "currency_id": ugx,
        "payment_method": "BANK_TRANSFER",
        "reference": "RP-001",
        "debt_payable_account_id": loan_liab,
        "int_accrd_or_exp_acc_id": int_accrued,
        "cash_account_id": cash
    });
    if let Some(sid) = schedule_id {
        repay_body["schedule_id"] = json!(sid);
    }

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/repayments"),
            repay_body,
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "repayment: {s} {body}");
    println!("✓ repayment principal=5m + interest");

    // ----- 8. Optional: fee + covenant (light) -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/fees"),
            json!({
                "fee_type": "COMMITMENT",
                "fee_date": "2025-01-10",
                "amount": "125000",   // 0.5% * undrawn 40m / 4 for quarter-ish
                "currency_id": ugx,
                "description": "Commitment fee on undrawn"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    // fee may or may not post GL yet — allow success or soft fail message
    if ok(s) {
        println!("✓ commitment fee");
    } else {
        println!("⚠ fee endpoint: {s} {body} (non-blocking)");
    }

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/capital/facilities/{facility_id}/covenants"),
            json!({
                "covenant_code": "DSCR-MIN",
                "covenant_name": "Minimum DSCR",
                "covenant_type": "FINANCIAL",
                "metric": "DSCR",
                "operator": "GE",
                "threshold_min": "1.25",
                "measurement_frequency": "QUARTERLY",
                "effective_date": "2025-01-10",
                "status": "ACTIVE"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "covenant: {s} {body}");
    println!("✓ DSCR covenant");

    // ----- 9. Read-backs -----
    let (s, body) = ctx
        .client
        .get(&format!("/api/capital/facilities/{facility_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get facility: {s} {body}");
    println!("✓ get facility");

    let (s, body) = ctx
        .client
        .get("/api/capital/facilities")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list facilities: {s} {body}");
    assert!(
        body["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 1,
        "expected ≥1 facility"
    );

    // ----- 10. Trial balance -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after debt lifecycle");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ debt_facility_lifecycle OK for {}", ctx.company_name);
    Ok(())
}

/*mod common;
use common::{E2eResult, TestContext};

#[tokio::test]
async fn raise_equity() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.raise_capital("20000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;

    let txn_id = ctx
        .raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .get(&format!("/api/transactions/get/{txn_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(s.is_success(), "get journal: {body}");

    // optional: list brief contains this uuid
    let (s, _list) = ctx.client.get("/api/transactions/list/brief").await?;
    assert!(s.is_success());
    Ok(())
}
*/
