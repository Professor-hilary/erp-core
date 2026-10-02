mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn financial_statements_smoke() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.raise_capital("25_000_000").await?;
    ctx.set_opening_balances().await?;

    for path in [
        "/api/reports/trial-balance?as_of=2025-01-31",
        "/api/reports/balancesheet/single?as_of=2025-01-31",
        "/api/reports/income?from=2025-01-01&to=2025-01-31",
        "/api/reports/cashflow/indirect?from=2025-01-01&to=2025-01-31",
        "/api/reports/change-of-equity?from=2025-01-01&to=2025-01-31",
    ] {
        let (status, body) = ctx.client.get(path).await?;
        assert!(
            status.is_success(),
            "report {path} failed ({status}): {body}"
        );
        println!("✓ {path}");
    }
    let _ = ctx.save_state();
    Ok(())
}
