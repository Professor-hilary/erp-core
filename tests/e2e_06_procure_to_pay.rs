mod common;
use common::TestContext;

use crate::common::E2eResult;

#[tokio::test]
#[ignore = "enable when procurement APIs are stable"]
async fn procure_to_pay() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.set_opening_balances().await?;
    ctx.seed_trading_partners().await?;

    // PO → GRN → bill → payment (paths per your handlers)
    // for vendor in ctx.vendors.values() { ... }
    let _ = ctx.save_state();

    Ok(())
}
