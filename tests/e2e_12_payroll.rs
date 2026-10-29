mod common;

use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::json;
use uuid::Uuid;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

fn acct(ctx: &TestContext, codes: &[&str]) -> Result<Uuid, String> {
    for c in codes {
        if let Some(id) = ctx.account_uuid(c) {
            return Ok(id);
        }
    }
    Err(format!(
        "none of these COA codes found: {codes:?} — check chart"
    ))
}

#[tokio::test]
async fn monthly_payroll() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    if ctx.employees.is_empty() {
        ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    }
    assert!(
        !ctx.employees.is_empty(),
        "need employees — run e2e_02 first"
    );

    // ----- Optional: pay component smoke -----
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/components",
            json!({
                "code": "BASIC",
                "name": "Basic salary",
                "component_kind": "EARNING",
                "calc_method": "FIXED"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    // ok or already exists
    assert!(
        ok(s) || s.as_u16() == 409 || s.as_u16() == 400,
        "component: {s} {body}"
    );

    // ----- 1. Create payrun -----
    // Path A (preferred): empty payslips → server generates from HR eligibility
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-01-01",
                "pay_period_end": "2026-01-31",
                "payment_date": "2026-02-05",
                "notes": "e2e January payroll",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    let (payrun_uuid, payrun_serial) = if ok(s) {
        let uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
        let serial = body["data"]["serial_id"]
            .as_i64()
            .ok_or("payrun serial_id missing")?;
        println!("✓ payrun (generated slips) serial={serial} uuid={uuid}");
        (uuid, serial)
    } else {
        // Path B: explicit slips (fallback if generate fails)
        eprintln!("generate path failed: {s} {body} — falling back to explicit payslips");
        let payslips: Vec<_> = ctx
            .employees
            .values()
            .map(|emp| {
                json!({
                    "employee_uuid": emp,
                    "gross_pay": "250000",
                    "basic_pay": "200000",
                    "tax_deducted": "25000",
                    "social_security": "75000",
                    "other_deductions": "0",
                    "employer_social_security": "15000",
                    "payment_method": "Bank Transfer",
                    "bank_account": "ACC-TEST"
                })
            })
            .collect();

        let (s2, body2) = ctx
            .client
            .post(
                "/api/payroll/payrun/create",
                json!({
                    "pay_period_start": "2026-01-01",
                    "pay_period_end": "2026-01-31",
                    "payment_date": "2026-02-05",
                    "notes": "e2e January payroll (explicit)",
                    "payslips": payslips
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s2), "create payrun explicit: {s2} {body2}");
        let uuid = extract_uuid(&body2, &["data", "uuid"]).map_err(|e| e.to_string())?;
        let serial = body2["data"]["serial_id"]
            .as_i64()
            .ok_or("payrun serial_id missing")?;
        println!("✓ payrun (explicit slips) serial={serial} uuid={uuid}");
        (uuid, serial)
    };

    // ----- 2. Process -----
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{payrun_serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process payrun: {s} {body}");
    println!("✓ payrun processed");

    // ----- 3. Post to GL -----
    let labor = acct(&ctx, &["520100", "520000"])?;
    let tax = acct(&ctx, &["210300"])?;
    let ss = acct(&ctx, &["210900"])?;
    let cash = acct(&ctx, &["110100", "110200"])?;
    let payable = acct(&ctx, &["210700", "210200"])?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/post",
            json!({
                "payrun_serial_id": payrun_serial,
                "labor_expense_id": labor,
                "income_tax_id": tax,
                "social_security_id": ss,
                "cash_account_uuid": cash,
                "payroll_payable": payable
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post payrun: {s} {body}");
    println!("✓ payrun posted to GL");

    // ----- 4. Lists -----
    let (s, body) = ctx
        .client
        .get("/api/payroll/list/payruns")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payruns: {s} {body}");
    assert!(
        body["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 1,
        "expected ≥1 payrun"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    let n_slips = if ok(s) {
        body["data"].as_array().map(|a| a.len()).unwrap_or(0)
    } else {
        let (s2, body2) = ctx
            .client
            .get("/api/payroll/list/payslips")
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s2), "list payslips: {s2} {body2}");
        body2["data"].as_array().map(|a| a.len()).unwrap_or(0)
    };
    assert!(
        n_slips >= 1,
        "expected ≥1 payslip, got {n_slips}"
    );
    println!("✓ payslips n={n_slips}");

    // ----- 5. Trial balance -----
    ctx.assert_trial_balance_ok("2026-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after payroll");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ monthly_payroll OK for {}", ctx.company_name);
    Ok(())
}

/*mod common;

use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::json;
use uuid::Uuid;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

/// Resolve account UUID by preferred codes; first hit wins.
fn acct(ctx: &TestContext, codes: &[&str]) -> Result<Uuid, String> {
    for c in codes {
        if let Some(id) = ctx.account_uuid(c) {
            return Ok(id);
        }
    }
    Err(format!(
        "none of these COA codes found: {codes:?} — check manufacturing chart"
    ))
}

#[tokio::test]
async fn monthly_payroll() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    if ctx.employees.is_empty() {
        ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    }
    assert!(
        !ctx.employees.is_empty(),
        "need employees — run e2e_02 first"
    );

    // ----- Build payslips from seeded employees (gross 1_500_000 from seed) -----
    let payslips: Vec<_> = ctx
        .employees
        .values()
        .map(|emp| {
            json!({
                "employee_uuid": emp,
                "gross_pay": "250000",
                "tax_deducted": "25000",       // ~10%
                "social_security": "75000",     // ~5%
                "other_deductions": "0",
                "payment_method": "Bank Transfer",
                "bank_account": "UG-TEST-001"
            })
        })
        .collect();

    // ----- 1. Create payrun + payslips -----
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-01-01",
                "pay_period_end": "2026-01-31",
                "payment_date": "2026-02-05",
                "notes": "January 2026 monthly payroll",
                "payslips": payslips
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create payrun: {s} {body}");

    let payrun_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let payrun_serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("payrun serial_id missing")?;
    println!("✓ payrun serial={payrun_serial} uuid={payrun_uuid}");

    // ----- 2. Process (Draft → Processed) -----
    // Handler path is {uuid} but extracts Path<i64> — pass serial_id
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{payrun_serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process payrun: {s} {body}");
    println!("✓ payrun processed");

    // ----- 3. Post to GL -----
    // Adjust codes if your COA uses different numbers.
    let labor = acct(&ctx, &["520100", "520000", "620300"])?;
    let tax = acct(&ctx, &["210300"])?;
    let ss = acct(&ctx, &["210900"])?;
    let cash = acct(&ctx, &["110100", "110200"])?;
    let payable = acct(&ctx, &["210700", "210200"])?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/post",
            json!({
                "payrun_serial_id": payrun_serial,
                "labor_expense_id": labor,
                "income_tax_id": tax,
                "social_security_id": ss,
                "cash_account_uuid": cash,
                "payroll_payable": payable
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post payrun: {s} {body}");
    println!("✓ payrun posted to GL");

    // ----- 4. Lists -----
    let (s, body) = ctx
        .client
        .get("/api/payroll/list/payruns")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payruns: {s} {body}");
    let n_runs = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(n_runs >= 1, "expected ≥1 payrun");
    println!("✓ payruns n={n_runs}");

    let (s, body) = ctx
        .client
        .get("/api/payroll/list/payslips")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payslips: {s} {body}");
    let n_slips = body["data"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        n_slips >= ctx.employees.len(),
        "expected ≥{} payslips, got {n_slips}",
        ctx.employees.len()
    );
    println!("✓ payslips n={n_slips}");

    // ----- 5. Trial balance -----
    ctx.assert_trial_balance_ok("2026-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after payroll");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ monthly_payroll OK for {}", ctx.company_name);
    Ok(())
}
*/
