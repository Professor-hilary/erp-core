mod common;
use common::TestContext;

use crate::common::E2eResult;

#[tokio::test]
#[ignore = "enable when manufacturing APIs are stable"]
async fn manufacturing_flow() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.set_opening_balances().await?;
    ctx.seed_workforce().await?;

    // TODO: create items, BOM, work order, receive FG
    // Example shape:
    // ctx.client.post("/api/manufacturing/items", json!({...})).await?;
    // ctx.client.post("/api/manufacturing/boms", json!({...})).await?;
    // ctx.client.post("/api/manufacturing/work-orders", json!({...})).await?;
    let _ = ctx.save_state();

    Ok(())
}
