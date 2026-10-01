mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn opening_balances() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;
    ctx.raise_capital("30_000_000").await?;
    let opening = ctx.set_opening_balances().await?;
    ctx.assert_trial_balance_ok("2025-01-01").await?;
    println!("✓ opening {opening}");
    Ok(())
}