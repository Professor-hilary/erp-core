mod common;
use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when payroll APIs are stable"]
async fn monthly_payroll() -> Result<()> {
    let mut ctx = TestContext::bootstrap_sme("manufacturing").await?;
    ctx.seed_workforce().await?;
    // run payrun for emp_* keys in ctx.employees
    Ok(())
}