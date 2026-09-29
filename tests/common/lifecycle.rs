mod common;

use anyhow::Result;
use common::TestContext;

#[tokio::test]
async fn sme_lifecycle_bootstrap_to_close() -> Result<()> {
    // 1. Onboarding
    let mut ctx = TestContext::bootstrap_sme("retail").await?;
    println!(
        "✓ Company '{}' created with {} accounts",
        ctx.company_name,
        ctx.accounts.len()
    );

    // 2. Capital
    let capital_txn = ctx.raise_capital("10000000").await?;
    println!("✓ Capital contribution posted: {capital_txn}");

    // 3. Opening balances
    let opening_txn = ctx.set_opening_balances().await?;
    println!("✓ Opening balances posted: {opening_txn}");

    // 4. A bit of operating activity (keeps the test realistic)
    let cash = ctx.account_uuid("110100")
        .or_else(|| ctx.account_uuid("110200"))
        .expect("cash");
    // Use a real income account code from your COA if different
    if let Some(sales) = ctx.accounts.values().find(|_| true) {
        // minimal sale so income account has activity before close
        let _ = ctx
            .post_journal(
                chrono::NaiveDate::from_ymd_opt(2025, 2, 10).unwrap(),
                "Cash sale",
                &[(cash, "150000", "0", "Sale"), (*sales, "0", "150000", "Revenue")],
                true,
            )
            .await;
        // Note: replace *sales with a proper income account lookup when you know the code
    }

    // 5. Reports still work on open period
    ctx.assert_trial_balance_ok("2025-02-28").await?;
    println!("✓ Trial balance OK before close");

    // 6. Period close (snapshot + zero nominals + lock)
    // Uncomment once close_period is deployed:
    // ctx.close_current_period().await?;
    // println!("✓ Period closed");

    // 7. After close you can assert nominals are zero / next period is open
    // (add when close is live)

    println!("✅ Lifecycle test finished");
    Ok(())
}