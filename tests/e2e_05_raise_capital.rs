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
				"facility_id": facility_id,
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
				"facility_id": facility_id,
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
				"facility_id": facility_id,
                "sequence_no": 1,
                "due_date": "2025-04-15",
                "principal_due": "5000000",
                "interest_due": interest_amt,
                "fee_due": "0",
                "status": "pending"
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
		"facility_id": facility_id,
		"schedule_id": schedule_id,
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
				"facility_id": facility_id,
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
				"facility_id": facility_id,
                "covenant_code": "DSCR-MIN",
                "covenant_name": "Minimum DSCR",
                "covenant_type": "FINANCIAL",
                "metric": "DSCR",
                "operator": ">=",
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

/// Proper equity stack: party → share class → shareholder → issue shares (instrument + GL).
#[tokio::test]
async fn equity_share_issue_lifecycle() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let ugx = ctx
        .currency_id("UGX")
        .ok_or_else(|| "UGX not seeded — run e2e_02".to_string())?;
    let cash = acct_uuid(&ctx, &["110200", "110100"])?;
    let share_cap = acct_uuid(&ctx, &["310100", "310000"])?;
    // Premium optional — only pass if present
    let share_prem = ctx.account_uuid("310500"); // reval reserve is a stand-in; omit if wrong

    // ----- 1. Ensure / create founder party -----
    let (s, parties) = ctx
        .client
        .get("/api/capital/party")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list parties: {s} {parties}");

    let founder = parties["data"]
        .as_array()
        .ok_or("parties not array")?
        .iter()
        .find(|p| {
            matches!(
                p["party_type"].as_str(),
                Some("SHAREHOLDER") | Some("INVESTOR")
            ) || matches!(p["role"].as_str(), Some("owner") | Some("shareholder"))
        });

    let party_id = if let Some(p) = founder {
        extract_uuid(p, &["id"])
            .or_else(|_| extract_uuid(p, &["uuid"]))
            .map_err(|e| e.to_string())?
    } else {
        let (s, body) = ctx
            .client
            .post(
                "/api/capital/party",
                json!({
                    "legal_name": "Hilary Founder",
                    "short_name": "Hilary",
                    "party_type": "SHAREHOLDER",
                    "role": "owner",
                    "country_code": "UG",
                    "is_related_party": true
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "create party: {s} {body}");
        extract_uuid(&body, &["data", "id"])
            .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
            .map_err(|e| e.to_string())?
    };
    println!("✓ party {party_id}");

    // Second investor party (Pearl Capital style)
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/party",
            json!({
                "legal_name": "Pearl Capital Partners",
                "short_name": "Pearl",
                "party_type": "INVESTOR",
                "role": "investor",
                "registration_number": "TIN-PCP-005",
                "country_code": "UG",
                "is_related_party": false
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create investor party: {s} {body}");
    let investor_party = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ investor party {investor_party}");

    // ----- 2. Share class (ordinary) -----
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/share-classes",
            json!({
                "class_code": "ORD-A",
                "name": "Ordinary Class A",
                "share_type": "ORDINARY",
                "authorized_shares": 1_000_000,
                "par_value": "1000",
                "currency_id": ugx,
                "voting_rights": true,
                "votes_per_share": "1",
                "dividend_rights": true,
                "dividend_preference": null,
                "is_callable": false,
                "is_convertible": false
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "share class: {s} {body}");
    let class_id = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ share class ORD-A {class_id}");

    // ----- 3. Shareholders -----
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/shareholders",
            json!({
                "party_id": party_id,
                "name": "Hilary Founder",
                "shareholder_type": "INDIVIDUAL",
                "residency_country": "UG",
                "is_related_party": true
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "shareholder founder: {s} {body}");
    let sh_founder = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ shareholder founder {sh_founder}");

    let (s, body) = ctx
        .client
        .post(
            "/api/capital/shareholders",
            json!({
                "party_id": investor_party,
                "name": "Pearl Capital Partners",
                "shareholder_type": "CORPORATE",
                "tax_id": "TIN-PCP-005",
                "residency_country": "UG",
                "is_related_party": false
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "shareholder investor: {s} {body}");
    let sh_investor = extract_uuid(&body, &["data", "id"])
        .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ shareholder investor {sh_investor}");

    // ----- 4. Issue shares (founder: 50k @ 1,000 = 50m) -----
    let issue1 = json!({
        "share_class_id": class_id,
        "shareholder_id": sh_founder,
        "shares": 50_000,
        "price_per_share": "1000",
        "issue_date": "2025-01-08",
        "currency_id": ugx,
        "cash_account_code": cash,
        "share_capital_code": share_cap,
        "par_value": "1000",
        "instrument_code": "EQ-ORD-FOUNDER-1",
        "instrument_name": "Founder ordinary issue",
        "reference": "ISSUE-001",
        "notes": "Seed equity founder"
    });
    // only attach premium if we resolved a uuid (and you trust the mapping)
    let _ = share_prem; // leave premium null for par-issue

    let (s, body) = ctx
        .client
        .post("/api/capital/share-transactions", issue1)
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue founder shares: {s} {body}");
    let inst1 = extract_uuid(&body, &["data", "instrument_id"])
        .or_else(|_| extract_uuid(&body, &["data", "id"]))
        .ok();
    println!("✓ issue founder 50k shares inst={inst1:?}");

    // ----- 5. Issue shares (investor: 20k @ 1,500 = 30m, premium 500/sh) -----
    // Without a dedicated share-premium GL, post full amount to share capital
    // or split if you add a premium account later.
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/share-transactions",
            json!({
                "share_class_id": class_id,
                "shareholder_id": sh_investor,
                "shares": 20_000,
                "price_per_share": "1500",
                "issue_date": "2025-01-12",
                "currency_id": ugx,
                "cash_account_code": cash,
                "share_capital_code": share_cap,
				"share_premium_code": share_prem,
                "par_value": "1000",
                "instrument_code": "EQ-ORD-PEARL-1",
                "instrument_name": "Pearl Seed Round",
                "reference": "ISSUE-002",
                "notes": "Seed investor at premium"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "issue investor shares: {s} {body}");
    println!("✓ issue investor 20k shares");

    // ----- 6. Standalone capital instrument (optional hybrid note) -----
    let (s, body) = ctx
        .client
        .post(
            "/api/capital/instruments",
            json!({
                "instrument_code": "EQ-SAFE-001",
                "name": "Seed SAFE placeholder",
                "instrument_family": "EQUITY",
                "instrument_type": "COMMON_EQUITY",
                "currency_id": ugx,
                "original_principal": "0",
                "face_value": "0",
                "issue_price": "0",
                "effective_date": "2025-01-12",
                "is_perpetual": true,
                "status": "ACTIVE",
                "notes": "Registry-only instrument for SAFE tracking"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    if ok(s) {
        let iid = extract_uuid(&body, &["data", "id"])
            .or_else(|_| extract_uuid(&body, &["data", "uuid"]))
            .ok();
        println!("✓ instrument SAFE {iid:?}");
    } else {
        println!("⚠ instrument create: {s} {body}");
    }

    // ----- 7. Read-backs -----
    for path in [
        "/api/capital/share-classes",
        "/api/capital/shareholders",
        "/api/capital/share-transactions",
        "/api/capital/instruments",
        "/api/capital/party",
    ] {
        let (s, body) = ctx
            .client
            .get(path)
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "{path}: {s} {body}");
        let n = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
        println!("✓ {path} n={n}");
    }

    let (s, body) = ctx
        .client
        .get(&format!("/api/capital/share-classes/{class_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get share class: {s} {body}");
    println!("✓ get share class");

    // ----- 8. TB -----
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after equity issue");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!(
        "✅ equity_share_issue_lifecycle OK for {}",
        ctx.company_name
    );
    Ok(())
}
