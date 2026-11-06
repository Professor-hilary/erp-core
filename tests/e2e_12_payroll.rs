//! Payroll e2e — full lifecycle against the calculation engine.
//!
//! Requires prior phases (onboarding + stakeholders) so COA and employees exist.
//! Run with: cargo test --test e2e_12_payroll -- --nocapture --test-threads=1

mod common;

use bigdecimal::BigDecimal;
use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::{Value, json};
use std::str::FromStr;
use uuid::Uuid;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

/// Parse a JSON amount that may be a string or a number.
fn json_amount(v: &Value) -> BigDecimal {
    if let Some(s) = v.as_str() {
        return BigDecimal::from_str(s.trim()).unwrap_or_else(|_| BigDecimal::from(0));
    }
    if let Some(n) = v.as_f64() {
        return BigDecimal::from_str(&format!("{n:.4}")).unwrap_or_else(|_| BigDecimal::from(0));
    }
    if let Some(i) = v.as_i64() {
        return BigDecimal::from(i);
    }
    BigDecimal::from(0)
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

/// Ensure a postable leaf account exists; create via `/api/accounts/create` if missing.
async fn ensure_account(
    ctx: &mut TestContext,
    code: &str,
    name: &str,
    category: &str,
    normal_balance: &str,
    parent_code: &str,
) -> Result<Uuid, String> {
    if let Some(id) = ctx.account_uuid(code) {
        return Ok(id);
    }

    let (s, body) = ctx
        .client
        .post(
            "/api/accounts/create",
            json!({
                "name": name,
                "category": category,
                "code": code,
                "normal_balance": normal_balance,
                "parent_code": parent_code,
                "is_contra": false,
                "cash_flow_category": null
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

    if ok(s) {
        let id = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
        ctx.accounts.insert(code.to_string(), id);
        println!("✓ created COA account {code} ({name})");
        return Ok(id);
    }

    // Already exists / validation race — reload from list
    if s.as_u16() == 409 || s.as_u16() == 400 {
        let (ls, list) = ctx
            .client
            .get("/api/accounts/list")
            .await
            .map_err(|e| e.to_string())?;
        if ok(ls) {
            if let Some(arr) = list["data"].as_array() {
                for a in arr {
                    if a["code"].as_str() == Some(code) {
                        if let Some(u) = a["uuid"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
                            ctx.accounts.insert(code.to_string(), u);
                            return Ok(u);
                        }
                    }
                }
            }
        }
    }

    Err(format!("failed to ensure account {code}: {s} {body}"))
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

struct PayrollAccounts {
    labor: Uuid,
    paye: Uuid,
    nssf_ee: Uuid,
    nssf_er: Uuid,
    nssf_er_expense: Uuid,
    payable: Uuid,
    cash: Uuid,
    other_deductions: Uuid,
    other_employer: Uuid,
}

async fn payroll_accounts(ctx: &mut TestContext) -> Result<PayrollAccounts, String> {
    let labor = match acct(ctx, &["520100", "520000"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(ctx, "520100", "Factory Wages", "expense", "dr", "520000").await?
        }
    };
    let paye = match acct(ctx, &["210300"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(ctx, "210300", "Taxes Payable", "liability", "cr", "210000").await?
        }
    };
    let nssf_ee = match acct(ctx, &["210910", "210900", "210800"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(
                ctx,
                "210910",
                "Employee NSSF Payable",
                "liability",
                "cr",
                "210000",
            )
            .await?
        }
    };
    let nssf_er = match acct(ctx, &["210920", "210900", "210800"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(
                ctx,
                "210920",
                "Employer NSSF Payable",
                "liability",
                "cr",
                "210000",
            )
            .await?
        }
    };
    let nssf_er_expense = match acct(ctx, &["520300", "520200", "520100"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(
                ctx,
                "520300",
                "Payroll Taxes - Employer",
                "expense",
                "dr",
                "520000",
            )
            .await?
        }
    };
    let payable = match acct(ctx, &["210200", "210700"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(ctx, "210200", "Accrued Wages", "liability", "cr", "210000").await?
        }
    };
    let cash = match acct(ctx, &["110100", "110200"]) {
        Ok(id) => id,
        Err(_) => ensure_account(ctx, "110100", "Cash on Hand", "asset", "dr", "110000").await?,
    };
    let other_deductions = match acct(ctx, &["210930", "210900", "210400"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(
                ctx,
                "210930",
                "Other Payroll Deductions Payable",
                "liability",
                "cr",
                "210000",
            )
            .await?
        }
    };
    let other_employer = match acct(ctx, &["520400", "520300", "520100"]) {
        Ok(id) => id,
        Err(_) => {
            ensure_account(
                ctx,
                "520400",
                "Other Employer Payroll Costs",
                "expense",
                "dr",
                "520000",
            )
            .await?
        }
    };

    Ok(PayrollAccounts {
        labor,
        paye,
        nssf_ee,
        nssf_er,
        nssf_er_expense,
        payable,
        cash,
        other_deductions,
        other_employer,
    })
}

fn assert_reasonable_payroll_amounts(slip: &Value) {
    let gross = json_amount(&slip["gross_pay"]);
    let paye = json_amount(&slip["tax_deducted"]);
    let nssf_ee = json_amount(&slip["social_security"]);
    let nssf_er = json_amount(slip.get("employer_social_security").unwrap_or(&Value::Null));
    let net = json_amount(slip.get("net_pay").unwrap_or(&Value::Null));
    let other = json_amount(slip.get("other_deductions").unwrap_or(&Value::Null));

    assert!(gross > BigDecimal::from(0), "expected positive gross, got {gross}");
    assert!(
        gross >= BigDecimal::from(100_000),
        "gross {gross} looks too low for a full-time contract"
    );
    assert!(paye > BigDecimal::from(0), "expected PAYE > 0, got {paye}");
    assert!(
        nssf_ee > BigDecimal::from(0),
        "expected employee NSSF > 0, got {nssf_ee}"
    );
    assert!(
        nssf_er > BigDecimal::from(0),
        "expected employer NSSF > 0, got {nssf_er}"
    );
    assert!(
        net > BigDecimal::from(0) && net < gross,
        "expected 0 < net < gross (net={net}, gross={gross})"
    );
    let recon = &net + &paye + &nssf_ee + &other;
    let diff = if &gross >= &recon {
        &gross - &recon
    } else {
        &recon - &gross
    };
    assert!(
        diff <= BigDecimal::from(2),
        "payslip identity failed: gross {gross} vs net+paye+nssf+other {recon} (diff {diff})"
    );
}

#[tokio::test]
async fn payroll_components_and_assignments() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;
    let emp = first_employee(&ctx)?;

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
        ok(s) || s.as_u16() == 409,
        "create housing component must succeed or already exist: {s} {body}"
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
        .ok_or("HOUSING component uuid missing after create/list")?;

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/assignments",
            json!({
                "employee_uuid": emp,
                "component_uuid": housing_uuid,
                "amount": "150000",
                "effective_from": "2026-01-01"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        ok(s) || s.as_u16() == 409,
        "create assignment must succeed or already exist: {s} {body}"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/assignments?employee_uuid={emp}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list assignments: {s} {body}");
    let rows = body["data"].as_array().cloned().unwrap_or_default();
    assert!(
        !rows.is_empty(),
        "expected at least one assignment listed for employee"
    );
    println!("✓ payroll_components_and_assignments OK");
    Ok(())
}

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
    assert_eq!(
        body["data"]["status"].as_str().unwrap_or(""),
        "Pending",
        "new payrun must be Pending"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list payslips for payrun: {s} {body}");
    let slips = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!slips.is_empty(), "prepare must seed ≥1 payslip");
    assert_eq!(
        json_amount(&slips[0]["gross_pay"]),
        BigDecimal::from(0),
        "shell payslip gross should be zero before process"
    );
    println!("✓ prepare payrun {payrun_uuid} slips={}", slips.len());
    ctx.save_state().map_err(|e| e.to_string())?;
    Ok(())
}

#[tokio::test]
async fn payroll_process_calculates_and_locks() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;

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
        .post(&format!("/api/payroll/payrun/process/{serial}"), json!({}))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process payrun: {s} {body}");
    assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "Processed");

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "payslips after process: {s} {body}");
    let slips = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!slips.is_empty());

    for slip in &slips {
        assert!(!slip["calculated_at"].is_null(), "missing calculated_at");
        assert!(
            !slip["calculation_version"].as_str().unwrap_or("").is_empty(),
            "missing calculation_version"
        );
        assert_reasonable_payroll_amounts(slip);

        if let Some(uuid) = slip["uuid"].as_str() {
            let (s2, items) = ctx
                .client
                .get(&format!("/api/payroll/payslip/{uuid}/calculation-items"))
                .await
                .map_err(|e| e.to_string())?;
            assert!(ok(s2), "calculation items: {s2} {items}");
            let arr = items["data"].as_array().cloned().unwrap_or_default();
            assert!(!arr.is_empty(), "expected calculation items");
            let codes: Vec<&str> = arr.iter().filter_map(|i| i["item_code"].as_str()).collect();
            assert!(
                codes.iter().any(|c| *c == "PAYE" || c.contains("TAX")),
                "expected PAYE line, got {codes:?}"
            );
        }
    }

    println!("✓ process serial={serial} slips={}", slips.len());
    ctx.save_state().map_err(|e| e.to_string())?;
    Ok(())
}

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
    if !ok(s) {
        println!("✓ create already rejected cross-month period: {s}");
        return Ok(());
    }
    let serial = body["data"]["serial_id"]
        .as_i64()
        .ok_or("serial_id missing")?;

    let (s, body) = ctx
        .client
        .post(&format!("/api/payroll/payrun/process/{serial}"), json!({}))
        .await
        .map_err(|e| e.to_string())?;
    assert!(!ok(s), "process must reject cross-month payrun, got {s} {body}");
    println!("✓ cross-month process rejected as expected");
    Ok(())
}

#[tokio::test]
async fn payroll_full_lifecycle() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_workforce(&mut ctx).await?;
    let accounts = payroll_accounts(&mut ctx).await?;

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

    let (s, body) = ctx
        .client
        .post(&format!("/api/payroll/payrun/process/{serial}"), json!({}))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "process: {s} {body}");
    assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "Processed");

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payslips"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "payslips: {s} {body}");
    let slips = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!slips.is_empty());
    for slip in &slips {
        assert_reasonable_payroll_amounts(slip);
    }

    let post_body = json!({
        "payrun_serial_id": serial,
        "labor_expense_id": accounts.labor,
        "income_tax_id": accounts.paye,
        "employee_social_security_id": accounts.nssf_ee,
        "employer_social_security_id": accounts.nssf_er,
        "employer_social_security_expense_id": accounts.nssf_er_expense,
        "payroll_payable": accounts.payable,
        "other_deductions_payable_id": accounts.other_deductions,
        "other_employer_cost_expense_id": accounts.other_employer
    });

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
    assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "Posted");
    assert!(!body["data"]["gl_transaction_uuid"].is_null());
    println!("✓ posted serial={serial}");

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
    assert_eq!(body["data"]["status"].as_str().unwrap_or(""), "PAID");

    let (s, body) = ctx
        .client
        .get(&format!("/api/payroll/payrun/{payrun_uuid}/payment-batch"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get payment batch: {s} {body}");
    println!("✓ employees settled");

    let (s, body) = ctx
        .client
        .post(
            "/api/payroll/remittances/create",
            json!({
                "payrun_uuid": payrun_uuid,
                "due_date": "2026-07-15",
                "statutory_code": ""
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create remittances: {s} {body}");
    let remittances = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!remittances.is_empty(), "expected PAYE/NSSF remittance rows");

    for rem in &remittances {
        let rem_uuid = rem["uuid"]
            .as_str()
            .and_then(|u| Uuid::parse_str(u).ok())
            .ok_or("remittance uuid")?;
        let code = rem["statutory_code"].as_str().unwrap_or("");
        assert!(
            json_amount(&rem["amount"]) > BigDecimal::from(0),
            "remittance {code} amount must be > 0"
        );
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

    ctx.assert_trial_balance_ok("2026-07-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ trial balance OK after full payroll lifecycle");
    ctx.save_state().map_err(|e| e.to_string())?;
    println!("✅ payroll_full_lifecycle OK for {}", ctx.company_name);
    Ok(())
}

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
        .post(&format!("/api/payroll/payrun/process/{serial}"), json!({}))
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
    assert!(!ok(s), "remittances must require Posted, got {s} {body}");
    println!("✓ remittance-before-post correctly rejected");
    Ok(())
}

#[tokio::test]
async fn payroll_list_surfaces() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let (s, body) = ctx.client.get("/api/payroll/list/payruns").await.map_err(|e| e.to_string())?;
    assert!(ok(s), "list payruns: {s} {body}");

    let (s, body) = ctx.client.get("/api/payroll/list/payslips").await.map_err(|e| e.to_string())?;
    assert!(ok(s), "list payslips: {s} {body}");

    let (s, body) = ctx.client.get("/api/payroll/remittances").await.map_err(|e| e.to_string())?;
    assert!(ok(s), "list remittances: {s} {body}");

    println!("✓ payroll list surfaces OK");
    Ok(())
}

#[tokio::test]
#[ignore = "duplicate of payroll_full_lifecycle; run that test instead"]
async fn monthly_payroll() -> E2eResult<()> {
    payroll_full_lifecycle().await
}
