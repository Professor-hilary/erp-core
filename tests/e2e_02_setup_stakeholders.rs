mod common;
use common::{E2eResult, TestContext, fixtures};
use serde_json::json;

fn ok(s: reqwest::StatusCode) -> bool {
    s.is_success() || s.as_u16() == 201
}

#[tokio::test]
async fn seed_all_stakeholders_and_institutions() -> E2eResult<()> {
    let mut ctx: TestContext = TestContext::from_saved_state().await?;

    ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    ctx.seed_currencies().await.map_err(|e| e.to_string())?;
    ctx.seed_capital_parties()
        .await
        .map_err(|e| e.to_string())?;
    ctx.seed_trading_partners()
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;

    assert!(fixtures::PEOPLE.len() >= 25);
    assert!(fixtures::INSTITUTIONS.len() >= 10);

    let (s, v) = ctx.client.get("/api/workforce/list/employees").await?;
    assert!(ok(s), "list employees: {s} {v}");
    let emps = v["data"].as_array().cloned().unwrap_or_default();
    assert!(!emps.is_empty(), "expected employees after seed");

    // Sample employee is Active + has contract
    let sample = ctx
        .employees
        .values()
        .next()
        .copied()
        .ok_or("no employee uuid in ctx")?;
    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/get/employee/{sample}"))
        .await?;
    assert!(ok(s), "get employee: {s} {body}");
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "Active", "seeded employee should be Active for payroll");

    let (s, body) = ctx
        .client
        .get(&format!(
            "/api/workforce/contracts?employee_uuid={sample}"
        ))
        .await?;
    assert!(ok(s), "list contracts: {s} {body}");
    let contracts = body["data"].as_array().cloned().unwrap_or_default();
    assert!(
        contracts.iter().any(|c| c["status"] == "Active"),
        "expected Active contract"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/employees/{sample}/payment-profile"))
        .await?;
    assert!(ok(s), "payment profile: {s} {body}");

    // Leave type (scheduled + encash — your workplace style)
    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/leave-types",
            json!({
                "code": "ANNUAL",
                "name": "Annual leave",
                "is_paid": true,
                "grant_mode": "SCHEDULED",
                "unused_treatment": "ENCASH",
                "default_days_per_year": "21"
            }),
        )
        .await?;
    // 201 or conflict if re-run
    assert!(
        ok(s) || s.as_u16() == 409 || s.as_u16() == 400,
        "leave type: {s} {body}"
    );
    println!("✓ leave type ANNUAL");

    let (s, _v) = ctx.client.get("/api/capital/party").await?;
    assert!(ok(s));

    let (s, _v) = ctx.client.get("/api/procurement/list/vendors").await?;
    assert!(ok(s));

    let (s, _v) = ctx.client.get("/api/sales/list/customers").await?;
    assert!(ok(s));

    assert!(ctx.currency_id("UGX").is_some(), "UGX must be seeded");

    println!(
        "✓ {} — employees={} vendors={} customers={} currencies n={}",
        ctx.company_name,
        ctx.employees.len(),
        ctx.vendors.len(),
        ctx.customers.len(),
        ctx.currencies.len()
    );
    Ok(())
}

/*mod common;
use common::{E2eResult, TestContext, fixtures};

#[tokio::test]
async fn seed_all_stakeholders_and_institutions() -> E2eResult<()> {
    let mut ctx: TestContext = TestContext::from_saved_state().await?;

    ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    ctx.seed_currencies().await.map_err(|e| e.to_string())?;
    ctx.seed_capital_parties()
        .await
        .map_err(|e| e.to_string())?;
    ctx.seed_trading_partners()
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;

    assert!(fixtures::PEOPLE.len() >= 25);
    assert!(fixtures::INSTITUTIONS.len() >= 10);

    let (s, v) = ctx.client.get("/api/workforce/list/employees").await?;
    assert!(s.is_success());
    assert!(
        v["data"].as_array().map(|a| a.len()).unwrap_or(0) >= 1,
        "expected employees after seed"
    );

    let (s, _v) = ctx.client.get("/api/capital/party").await?;
    assert!(s.is_success());

    let (s, _v) = ctx.client.get("/api/procurement/list/vendors").await?;
    assert!(s.is_success());

    let (s, _v) = ctx.client.get("/api/sales/list/customers").await?;
    assert!(s.is_success());

    assert!(ctx.currency_id("UGX").is_some(), "UGX must be seeded");

    println!(
        "✓ {} — employees={} vendors={} customers={} currencies n={}",
        ctx.company_name,
        ctx.employees.len(),
        ctx.vendors.len(),
        ctx.customers.len(),
        ctx.currencies.len()
    );
    Ok(())
}
*/
