mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn period_close() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("25_000_000").await?;
    ctx.set_opening_balances().await?;

    // small activity so close has nominal balances
    if let (Some(cash), Some(equity)) = (ctx.account_uuid("110200"), ctx.account_uuid("310000")) {
        // already have capital; optional expense/income journals here
        let _ = (cash, equity);
    }

    // Uncomment when close_period is deployed:
    // ctx.close_current_period().await?;

    ctx.assert_trial_balance_ok("2025-01-31").await?;
    let _ = ctx.save_state();
    Ok(())
}
