mod common;
use common::{E2eResult, TestContext};

#[tokio::test]
async fn onboarding_creates_company_and_coa() -> E2eResult<()> {
    let ctx = TestContext::bootstrap_sme("manufacturing")
        .await
        .map_err(|e| e.to_string())?;

    assert!(
        !ctx.accounts.is_empty(),
        "COA must be seeded for manufacturing"
    );
    assert!(
        ctx.account_uuid("310000").is_some(),
        "Owner's Equity required"
    );

    ctx.save_state()?;
    println!("✓ {} ({})", ctx.company_name, ctx.company_id);
    Ok(())
}
