mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when sales APIs are stable"]
async fn order_to_cash() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("40_000_000").await?;
    ctx.set_opening_balances().await?;
    ctx.seed_trading_partners().await?;

    // quote/order → delivery → invoice → receipt
    let _ = ctx.save_state();

    Ok(())
}
