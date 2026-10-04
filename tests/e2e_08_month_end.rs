mod common;
use common::TestContext;

use crate::common::E2eResult;

#[tokio::test]
async fn period_close() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    // ctx.set_opening_balances().await?;

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
