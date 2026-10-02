mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn opening_balances() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("30_000_000").await?;
    let opening = ctx.set_opening_balances().await?;
    ctx.assert_trial_balance_ok("2025-01-01").await?;
    let _ = ctx.save_state();
    println!("✓ opening {opening}");
    Ok(())
}
