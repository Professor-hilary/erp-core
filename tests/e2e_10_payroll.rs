mod common;
use std::error::Error;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
#[ignore = "enable when payroll APIs are stable"]
async fn monthly_payroll() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut ctx = TestContext::from_saved_state().await?;
    ctx.seed_workforce().await?;
    // run payrun for emp_* keys in ctx.employees
    let _ = ctx.save_state();
    Ok(())
}
