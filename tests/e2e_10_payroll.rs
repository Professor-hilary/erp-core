mod common;
use common::TestContext;

use crate::common::E2eResult;

#[tokio::test]
#[ignore = "enable when payroll APIs are stable"]
async fn monthly_payroll() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    // ctx.seed_workforce().await?;
    // run payrun for emp_* keys in ctx.employees
    let _ = ctx.save_state();
    Ok(())
}
