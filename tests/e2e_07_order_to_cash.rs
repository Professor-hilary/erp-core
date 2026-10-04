mod common;
use common::TestContext;

use crate::common::E2eResult;

#[tokio::test]
#[ignore = "enable when sales APIs are stable"]
async fn order_to_cash() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.set_opening_balances().await?;
    ctx.seed_trading_partners().await?;

    // quote/order → delivery → invoice → receipt
    let _ = ctx.save_state();

    Ok(())
}
