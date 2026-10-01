mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when sales APIs are stable"]
async fn order_to_cash() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;
    ctx.raise_capital("40_000_000").await?;
    ctx.set_opening_balances().await?;
    ctx.seed_trading_partners().await?;

    // quote/order → delivery → invoice → receipt

    Ok(())
}