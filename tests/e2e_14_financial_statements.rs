mod common;
use common::TestContext;

use crate::common::E2eResult;
use serde_json::Value;

/// Print the response exactly as the backend returned it (pretty JSON).
fn dump_backend(path: &str, body: &Value) {
    println!();
    println!("======== BACKEND {path} ========");
    match serde_json::to_string_pretty(body) {
        Ok(s) => println!("{s}"),
        Err(e) => println!("(failed to pretty-print: {e}) raw={body}"),
    }
    println!("======== END {path} ========");
}

#[tokio::test]
async fn financial_statements_smoke() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    for path in [
        "/api/reports/trial-balance?as_of=2026-12-31",
        "/api/reports/balancesheet/single?as_of=2026-12-31",
        "/api/reports/income?from=2026-01-01&to=2026-12-31",
        "/api/reports/cashflow/indirect?from=2026-01-01&to=2026-12-31",
        "/api/reports/change-of-equity?from=2026-01-01&to=2026-12-31",
    ] {
        let (status, body) = ctx.client.get(path).await?;
        assert!(
            status.is_success(),
            "report {path} failed ({status}): {body}"
        );
        dump_backend(path, &body);
        println!("✓ {path}");
    }

    println!(
        "✅ financial statements dumped for {} ({})",
        ctx.company_name, ctx.company_id
    );
    let _ = ctx.save_state();
    Ok(())
}
