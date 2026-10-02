mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn raise_equity_and_optional_debt() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("50_000_000").await?;
    ctx.raise_capital("20_000_000").await?; // Optional second contribution
    let _ = ctx.save_state();
    ctx.assert_trial_balance_ok("2025-01-31").await?;
    println!("✓ capital journals");
    Ok(())
}
