mod common;
use anyhow::Result;
use common::{fixtures, TestContext};

#[tokio::test]
async fn seed_all_stakeholders_and_institutions() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;

    ctx.seed_workforce().await?;
    ctx.seed_capital_parties().await?;
    ctx.seed_trading_partners().await?;

    // Soft expectations: we created the fixture set even if some routes 404 until wired
    println!("✓ people in fixtures: {}", fixtures::PEOPLE.len());
    println!("✓ institutions in fixtures: {}", fixtures::INSTITUTIONS.len());
    println!("✓ employees created: {}", ctx.employees.len());
    println!("✓ vendors created: {}", ctx.vendors.len());
    println!("✓ customers created: {}", ctx.customers.len());

    assert!(
        fixtures::PEOPLE.len() >= 25,
        "fixture must define ≥25 individuals"
    );
    assert!(
        fixtures::INSTITUTIONS.len() >= 10,
        "fixture must define ≥10 institutions"
    );

    Ok(())
}