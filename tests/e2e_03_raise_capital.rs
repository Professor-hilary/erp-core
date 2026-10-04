mod common;
use common::{E2eResult, TestContext};

#[tokio::test]
async fn raise_equity() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.raise_capital("20000000")
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;
    ctx.assert_trial_balance_ok("2025-01-31")
        .await
        .map_err(|e| e.to_string())?;

    let txn_id = ctx
        .raise_capital("50000000")
        .await
        .map_err(|e| e.to_string())?;

    let (s, body) = ctx
        .client
        .get(&format!("/api/transactions/get/{txn_id}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(s.is_success(), "get journal: {body}");

    // optional: list brief contains this uuid
    let (s, list) = ctx.client.get("/api/transactions/list/brief").await?;
    assert!(s.is_success());
    Ok(())
}
