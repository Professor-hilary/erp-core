//! Payroll e2e — full lifecycle against the calculation engine.
//!
//! Requires prior phases (onboarding + stakeholders) so COA and employees exist.
//! Run with: cargo test --test e2e_12_payroll -- --nocapture --test-threads=1

mod common;

use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::{Value, json};
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

async fn ensure_workforce(ctx: &mut TestContext) -> Result<(), String> {
    if ctx.employees.is_empty() {
        ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    }
    assert!(
        !ctx.employees.is_empty(),
        "need employees — run e2e_02 first or seed_workforce"
    );
    Ok(())
}

fn first_employee(ctx: &TestContext) -> Result<Uuid, String> {
    ctx.employees
        .values()
        .next()
        .copied()
        .ok_or_else(|| "no employee uuid in ctx".into())
}

/// Resolve the GL accounts used by post / settle / remit.
struct PayrollAccounts {
    labor: Uuid,
    paye: Uuid,
    nssf_ee: Uuid,
    nssf_er: Uuid,
    nssf_er_expense: Uuid,
    payable: Uuid,
    cash: Uuid,
    other_deductions: Option<Uuid>,
    other_employer: Option<Uuid>,
}

fn payroll_accounts(ctx: &TestContext) -> Result<PayrollAccounts, String> {
    Ok(PayrollAccounts {
        labor: acct(ctx, &["520100", "520000", "620300"])?,
        paye: acct(ctx, &["210300"])?,
        // Prefer distinct EE/ER liability codes when present; fall back to shared SS payable
        nssf_ee: acct(ctx, &["210900"])?,
        nssf_er: acct(ctx, &["210900"])?,
        nssf_er_expense: acct(ctx, &["520100", "520000"])?,
        payable: acct(ctx, &["210200"])?,
        cash: acct(ctx, &["110100", "110200"])?,
        other_deductions: ctx
            .account_uuid("210930")
            .or_else(|| ctx.account_uuid("210900")),
        other_employer: ctx
            .account_uuid("520300")
            .or_else(|| ctx.account_uuid("520100")),
    })
}

// =============================================================================
// 1. Components + assignments
// =============================================================================

#[tokio::test]
async fn payroll_components_and_assignments() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;
    let emp = first_employee(&ctx)?;

    // Create (or tolerate existing) earning component
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/components",
            json!({
                "code": "HOUSING",
                "name": "Housing allowance",
                "component_kind": "EARNING",
                "calc_method": "FIXED",
                "is_taxable": true,
                "affects_social_security": true,
                "sequence_no": 20,
                "rounding_scale": 2
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        ok(s) || s.as_u16() == 409 || s.as_u16() == 400,
        "create housing component: {s} {body}"
    );

    let (s, body) = ctx
        .client
        .get("/api/payroll/components")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list components: {s} {body}");
    let components = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!components.is_empty(), "expected ≥1 pay component");

    let housing_uuid = components
        .iter()
        .find(|c| c["code"].as_str() == Some("HOUSING"))
        .and_then(|c| c["uuid"].as_str())
        .and_then(|u| Uuid::parse_str(u).ok())
        .or_else(|| {
            components
                .iter()
                .find(|c| c["component_kind"].as_str() == Some("EARNING"))
                .and_then(|c| c["uuid"].as_str())
                .and_then(|u| Uuid::parse_str(u).ok())
        });

    if let Some(comp) = housing_uuid {
        let (s, body) = ctx
            .client
            .post(
                "/api/payroll/assignments",
                json!({
                    "employee_uuid": emp,
                    "component_uuid": comp,
                    "amount": "150000",
                    "effective_from": "2026-01-01"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(
            ok(s) || s.as_u16() == 409,
            "create assignment: {s} {body}"
        );
        println!("✓ assignment created for employee {emp}");
    }

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/assignments?employee_uuid={emp}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list assignments: {s} {body}");
    println!("✓ payroll_components_and_assignments OK");
    Ok(())
}

// =============================================================================
// 2. Prepare payrun (eligible employees → zeroed slips)
// =============================================================================

#[tokio::test]
async fn payroll_prepare_payrun() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-03-01",
                "pay_period_end": "2026-03-31",
                "payment_date": "2026-04-05",
                "notes": "e2e March prepare",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create payrun: {s} {body}");

    let payrun_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "Pending", "new payrun must be Pending");

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payslips for payrun: {s} {body}");
    let slips = body["data"].as_array().cloned().unwrap_or_default();
    assert!(
        !slips.is_empty(),
        "prepare must seed ≥1 payslip from eligible employees"
    );
    // Shells are zero until process
    let gross0 = slips[0]["gross_pay"]
        .as_str()
        .or_else(|| slips[0]["gross_pay"].as_f64().map(|_| "0"))
        .unwrap_or("0");
    println!(
        "✓ prepare payrun {payrun_uuid} slips={} first_gross={gross0}",
        slips.len()
    );
    ctx.save_state().map_err(|e| e.to_string())?;
    Ok(())
}

// =============================================================================
// 3. Process (calculate + lock)
// =============================================================================

#[tokio::test]
async fn payroll_process_calculates_and_locks() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;

    // Fresh payrun for process
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-04-01",
                "pay_period_end": "2026-04-30",
                "payment_date": "2026-05-05",
                "notes": "e2e April process",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create payrun: {s} {body}");
    let payrun_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("payrun serial_id missing")?;

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process payrun: {s} {body}");
    assert_eq!(
        body["data"]["status"].as_str().unwrap_or(""),
        "Processed",
        "payrun must be Processed after calculation"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "payslips after process: {s} {body}");
    let slips = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!slips.is_empty());

    for slip in &slips {
        let calculated_at = &slip["calculated_at"];
        assert!(
            !calculated_at.is_null(),
            "payslip must have calculated_at: {slip}"
        );
        let version = slip["calculation_version"].as_str().unwrap_or("");
        assert!(
            !version.is_empty(),
            "payslip must have calculation_version"
        );
        // Net is generated; gross should be positive for contracted staff
        let gross = slip["gross_pay"].as_str().unwrap_or("0");
        assert!(
            gross != "0" && gross != "0.00",
            "expected positive gross after process, got {gross}"
        );

        if let Some(uuid) = slip["uuid"].as_str() {
            let (s2, items) = ctx
                .client
                .get(&format!("/api/payroll/payslip/{uuid}/calculation-items"))
                .await
                .map_err(|e| e.to_string())?;
            assert!(ok(s2), "calculation items: {s2} {items}");
            let n = items["data"].as_array().map(|a| a.len()).unwrap_or(0);
            assert!(n >= 1, "expected calculation items for payslip {uuid}");
        }
    }

    println!(
        "✓ process serial={serial} slips={} all calculated",
        slips.len()
    );
    ctx.save_state().map_err(|e| e.to_string())?;
    Ok(())
}

// =============================================================================
// 4. Cross-month payrun must be rejected
// =============================================================================

#[tokio::test]
async fn payroll_rejects_cross_month_period() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-05-20",
                "pay_period_end": "2026-06-10",
                "payment_date": "2026-06-15",
                "notes": "cross-month should fail at process",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    // Create may succeed; process must fail
    if !ok(s) {
        println!("✓ create already rejected cross-month period: {s}");
        return Ok(());
    }
    let serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("serial_id missing")?;

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        !ok(s),
        "process must reject cross-month payrun, got {s} {body}"
    );
    println!("✓ cross-month process rejected as expected");
    Ok(())
}

// =============================================================================
// 5. Full happy path: prepare → process → post → settle → remit
// =============================================================================

#[tokio::test]
async fn payroll_full_lifecycle() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;
    let accounts = payroll_accounts(&ctx)?;

    // --- prepare ---
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-06-01",
                "pay_period_end": "2026-06-30",
                "payment_date": "2026-07-05",
                "notes": "e2e June full lifecycle",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create: {s} {body}");
    let payrun_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("serial_id missing")?;

    // --- process ---
    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process: {s} {body}");
    assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "Processed");

    // --- post accrual ---
    let mut post_body = json!({
        "payrun_serial_id": serial,
        "labor_expense_id": accounts.labor,
        "income_tax_id": accounts.paye,
        "employee_social_security_id": accounts.nssf_ee,
        "employer_social_security_id": accounts.nssf_er,
        "employer_social_security_expense_id": accounts.nssf_er_expense,
        "payroll_payable": accounts.payable
    });
    if let Some(id) = accounts.other_deductions {
        post_body["other_deductions_payable_id"] = json!(id);
    }
    if let Some(id) = accounts.other_employer {
        post_body["other_employer_cost_expense_id"] = json!(id);
    }

    let (s, body) = ctx
        .client
        .post("/api/payroll/payrun/post", post_body)
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "post: {s} {body}");

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/get/payrun/{payrun_uuid}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get payrun: {s} {body}");
    assert_eq!(
        body["data"]["status"].as_str().unwrap_or(""),
        "Posted",
        "payrun must be Posted after GL accrual"
    );
    assert!(
        !body["data"]["gl_transaction_uuid"].is_null(),
        "Posted payrun must carry gl_transaction_uuid"
    );
    println!("✓ posted serial={serial}");

    // --- settle employees ---
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/settle",
            json!({
                "payrun_uuid": payrun_uuid,
                "payment_date": "2026-07-05",
                "payment_reference": "E2E-JUNE-NET",
                "cash_account_uuid": accounts.cash,
                "payroll_payable_uuid": accounts.payable
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "settle: {s} {body}");
    assert_eq!(
        body["data"]["status"].as_str().unwrap_or(""),
        "PAID",
        "payment batch must be PAID"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payment-batch"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get payment batch: {s} {body}");
    println!("✓ employees settled");

    // --- statutory remittances (require Posted) ---
    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/remittances/create",
            json!({
                "payrun_uuid": payrun_uuid,
                "due_date": "2026-07-15"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create remittances: {s} {body}");
    let remittances = body["data"].as_array().cloned().unwrap_or_default();
    assert!(
        !remittances.is_empty(),
        "expected PAYE/NSSF remittance rows"
    );

    for rem in &remittances {
        let rem_uuid = rem["uuid"]
            .as_str()
            .and_then(|u| Uuid::parse_str(u).ok())
            .ok_or("remittance uuid")?;
        let code = rem["statutory_code"].as_str().unwrap_or("");
        let liability = match code {
            "PAYE" => accounts.paye,
            "NSSF_EE" => accounts.nssf_ee,
            "NSSF_ER" => accounts.nssf_er,
            _ => accounts.nssf_ee,
        };
        let (s, body) = ctx
            .client
            .post(
                "/api/payroll/remittances/pay",
                json!({
                    "remittance_uuid": rem_uuid,
                    "payment_date": "2026-07-15",
                    "liability_account_uuid": liability,
                    "cash_account_uuid": accounts.cash
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s), "remit {code}: {s} {body}");
        assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "PAID");
    }
    println!("✓ remittances paid n={}", remittances.len());

    // Trial balance after full cycle
    ctx.assert_trial_balance_ok("2026-07-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after full payroll lifecycle");

    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ payroll_full_lifecycle OK for {}", ctx.company_name);
    Ok(())
}

// =============================================================================
// 6. Remittances cannot be created before Post
// =============================================================================

#[tokio::test]
async fn payroll_remittance_requires_posted() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/payrun/create",
            json!({
                "pay_period_start": "2026-07-01",
                "pay_period_end": "2026-07-31",
                "payment_date": "2026-08-05",
                "notes": "remittance-before-post guard",
                "payslips": []
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create: {s} {body}");
    let payrun_uuid = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("serial_id missing")?;

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/payroll/payrun/process/{serial}"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/remittances/create",
            json!({
                "payrun_uuid": payrun_uuid,
                "due_date": "2026-08-15"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        !ok(s),
        "remittances must require Posted, got {s} {body}"
    );
    println!("✓ remittance-before-post correctly rejected");
    Ok(())
}

// =============================================================================
// 7. List surfaces
// =============================================================================

#[tokio::test]
async fn payroll_list_surfaces() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let (s, body) = ctx
        .client
        .get("/api/payroll/list/payruns")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payruns: {s} {body}");

    let (s, body) = ctx
        .client
        .get("/api/payroll/list/payslips")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payslips: {s} {body}");

    let (s, body) = ctx
        .client
        .get("/api/payroll/remittances")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list remittances: {s} {body}");

    println!("✓ payroll list surfaces OK");
    Ok(())
}

// Keep a thin alias so older docs that call monthly_payroll still resolve.
#[tokio::test]
async fn monthly_payroll() -> E2eResult<()> {
    // Delegate to the full lifecycle so a single entry point still exercises the engine.
    payroll_full_lifecycle().await
}

// Silence unused warning when Value is only needed in helpers.
#[allow(dead_code)]
fn _value_touch(v: &Value) -> bool {
    !v.is_null()
}
