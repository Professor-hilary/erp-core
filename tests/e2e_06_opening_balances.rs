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
    ctx.assert_trial_balance_ok("2026-01-01")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ opening {opening}");

    ctx.post_operating_costs()
        .await
        .map_err(|e| e.to_string())?;
    ctx.assert_trial_balance_ok("2026-01-31")
        .await
        .map_err(|e| e.to_string())?;
    println!("✓ period operating costs posted");
    Ok(())
}
