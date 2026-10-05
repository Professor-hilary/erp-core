mod common;
use common::{E2eResult, TestContext, fixtures};

#[tokio::test]
async fn seed_all_stakeholders_and_institutions() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;

    ctx.seed_workforce().await.map_err(|e| e.to_string())?;
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

    println!(
        "✓ {} — employees={} vendors={} customers={}",
        ctx.company_name,
        ctx.employees.len(),
        ctx.vendors.len(),
        ctx.customers.len()
    );
    Ok(())
}
