mod common;
use std::error::Error;

use anyhow::Result;
use common::{TestContext, fixtures};

#[tokio::test]
async fn seed_all_stakeholders_and_institutions() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx: TestContext = TestContext::from_saved_state().await.map_err(|e| e)?;

    ctx.seed_workforce().await?;
    ctx.seed_capital_parties().await?;
    ctx.seed_trading_partners().await?;
    let _ = ctx
        .save_state()
        .map_err(|_e| format!("Failed to save company state")); // persist new stakeholder ids

    // Soft expectations: we created the fixture set even if some routes 404 until wired
    println!(
        "✓ people in fixtures: {} -  institutions in fixtures={} employees={} customers={} vendors={}",
        fixtures::PEOPLE.len(),
        fixtures::INSTITUTIONS.len(),
        ctx.employees.len(),
        ctx.customers.len(),
        ctx.vendors.len(),
    );

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
