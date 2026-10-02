mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when manufacturing APIs are stable"]
async fn manufacturing_flow() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("40_000_000").await?;
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
