mod common;
use common::{E2eResult, TestContext};

#[tokio::test]
async fn opening_balances() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    let opening = ctx
        .set_opening_balances()
        .await
        .map_err(|e| e.to_string())?;
    ctx.save_state()?;
    ctx.assert_trial_balance_ok("2025-01-01")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ opening {opening}");
    Ok(())
}
