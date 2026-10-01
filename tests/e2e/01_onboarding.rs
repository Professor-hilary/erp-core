mod common;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn onboarding_creates_company_and_coa() -> Result<()> {
    let ctx = TestContext::bootstrap_sme("retail").await?;

    assert!(!ctx.accounts.is_empty(), "COA must be seeded");
    assert!(
        ctx.account_uuid("310000").is_some(),
        "Owner's Equity required"
    );
    assert!(
        ctx.account_uuid("110100").is_some() || ctx.account_uuid("110200").is_some(),
        "Cash/Bank required"
    );

    println!(
        "✓ {} — {} accounts, company {}",
        ctx.company_name,
        ctx.accounts.len(),
        ctx.company_id
    );
    Ok(())
}
