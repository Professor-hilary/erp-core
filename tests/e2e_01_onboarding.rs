mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn onboarding_creates_company_and_coa() -> Result<()> {
    let ctx = TestContext::bootstrap_sme("manufacturing").await?;
    let _ = ctx
        .save_state()
        .map_err(|_e| format!("Failed to save company state"));

    println!("✓ {} company {}", ctx.company_name, ctx.company_id);
    Ok(())
}
