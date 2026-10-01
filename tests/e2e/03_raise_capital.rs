mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn raise_equity_and_optional_debt() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;
    ctx.seed_capital_parties().await?;

    let t1 = ctx.raise_capital("50_000_000").await?;
    // Optional second contribution
    let t2 = ctx.raise_capital("20_000_000").await?;

    ctx.assert_trial_balance_ok("2025-01-31").await?;
    println!("✓ capital journals {t1}, {t2}");
    Ok(())
}