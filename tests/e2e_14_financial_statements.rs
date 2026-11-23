mod common;
use common::TestContext;

use crate::common::E2eResult;
use serde_json::Value;

/// Pull row arrays out of the common ApiResponse envelopes.
fn rows_of(body: &Value) -> &[Value] {
    if let Some(arr) = body["data"].as_array() {
        return arr.as_slice();
    }
    if let Some(arr) = body["data"]["rows"].as_array() {
        return arr.as_slice();
    }
    if let Some(arr) = body["data"]["items"].as_array() {
        return arr.as_slice();
    }
    if let Some(arr) = body.as_array() {
        return arr.as_slice();
    }
    &[]
}

fn cell(v: &Value, keys: &[&str]) -> String {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if x.is_null() {
                continue;
            }
            if let Some(s) = x.as_str() {
                return s.to_string();
            }
            return x.to_string();
        }
    }
    String::new()
}

fn print_statement(title: &str, body: &Value) {
    let rows = rows_of(body);
    println!();
    println!("======== {title} ({} rows) ========", rows.len());
    println!(
        "{:<10} {:<42} {:<12} {:>16}",
        "code", "name", "category", "balance"
    );
    println!("{}", "-".repeat(84));
    for r in rows {
        let code = cell(r, &["code", "account_code"]);
        let name = cell(r, &["name", "account_name", "description"]);
        let cat = cell(r, &["category", "section", "activity_section"]);
        let bal = cell(r, &["balance", "amount", "debit", "credit"]);
        let name_trim: String = name.chars().take(42).collect();
        println!("{code:<10} {name_trim:<42} {cat:<12} {bal:>16}");
    }
    println!("{}", "=".repeat(84));
}

fn sum_field(rows: &[Value], field: &str) -> f64 {
    rows.iter()
        .filter_map(|r| {
            r.get(field)
                .and_then(|v| v.as_str().map(|s| s.parse().ok()).flatten().or_else(|| v.as_f64()))
        })
        .sum()
}

#[tokio::test]
async fn financial_statements_smoke() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // --- Trial balance: print + soft balance check ---
    let path = "/api/reports/trial-balance?as_of=2026-12-31";
    let (status, body) = ctx.client.get(path).await?;
    assert!(status.is_success(), "report {path} failed ({status}): {body}");
    print_statement("TRIAL BALANCE as of 2026-12-31", &body);
    let rows = rows_of(&body);
    assert!(!rows.is_empty(), "trial balance returned no rows");
    // Prefer explicit debit/credit columns when present; else use signed balance.
    let debits = sum_field(rows, "debit");
    let credits = sum_field(rows, "credit");
    if debits > 0.0 || credits > 0.0 {
        let diff = (debits - credits).abs();
        println!("TB debit total={debits:.2} credit total={credits:.2} |diff|={diff:.2}");
        assert!(
            diff < 1.0,
            "trial balance out of balance: debit={debits} credit={credits}"
        );
    }
    println!("✓ {path}");

    // --- Balance sheet ---
    let path = "/api/reports/balancesheet/single?as_of=2026-12-31";
    let (status, body) = ctx.client.get(path).await?;
    assert!(status.is_success(), "report {path} failed ({status}): {body}");
    print_statement("BALANCE SHEET as of 2026-12-31", &body);
    assert!(!rows_of(&body).is_empty(), "balance sheet returned no rows");
    println!("✓ {path}");

    // --- Income statement ---
    let path = "/api/reports/income?from=2026-01-01&to=2026-12-31";
    let (status, body) = ctx.client.get(path).await?;
    assert!(status.is_success(), "report {path} failed ({status}): {body}");
    print_statement("INCOME STATEMENT 2026-01-01 .. 2026-12-31", &body);
    assert!(!rows_of(&body).is_empty(), "income statement returned no rows");
    println!("✓ {path}");

    // --- Cash flow indirect ---
    let path = "/api/reports/cashflow/indirect?from=2026-01-01&to=2026-12-31";
    let (status, body) = ctx.client.get(path).await?;
    assert!(status.is_success(), "report {path} failed ({status}): {body}");
    print_statement("CASH FLOW (indirect) 2026-01-01 .. 2026-12-31", &body);
    println!("✓ {path}");

    // --- Change of equity ---
    let path = "/api/reports/change-of-equity?from=2026-01-01&to=2026-12-31";
    let (status, body) = ctx.client.get(path).await?;
    assert!(status.is_success(), "report {path} failed ({status}): {body}");
    print_statement("CHANGE OF EQUITY 2026-01-01 .. 2026-12-31", &body);
    println!("✓ {path}");

    println!(
        "✅ financial statements generated for {} ({})",
        ctx.company_name, ctx.company_id
    );
    let _ = ctx.save_state();
    Ok(())
}
