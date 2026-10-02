mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when procurement APIs are stable"]
async fn procure_to_pay() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("40_000_000").await?;
    ctx.set_opening_balances().await?;
    ctx.seed_trading_partners().await?;

    // PO → GRN → bill → payment (paths per your handlers)
    // for vendor in ctx.vendors.values() { ... }
    let _ = ctx.save_state();

    Ok(())
}
