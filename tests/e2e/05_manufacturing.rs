mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when manufacturing APIs are stable"]
async fn manufacturing_flow() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;
    ctx.raise_capital("40_000_000").await?;
    ctx.set_opening_balances().await?;
    ctx.seed_workforce().await?;

    // TODO: create items, BOM, work order, receive FG
    // Example shape:
    // ctx.client.post("/api/manufacturing/items", json!({...})).await?;
    // ctx.client.post("/api/manufacturing/boms", json!({...})).await?;
    // ctx.client.post("/api/manufacturing/work-orders", json!({...})).await?;

    Ok(())
}