mod common;
use common::TestContext;

use crate::common::E2eResult;
use serde_json::Value;

fn data_array(body: &Value) -> &[Value] {
    body.get("data")
        .and_then(|d| d.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[])
}

fn s(v: &Value, key: &str) -> String {
    match v.get(key) {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(x)) => x.clone(),
        Some(other) => other.to_string(),
    }
}

fn parse_amount(raw: &str) -> f64 {
    raw.replace(',', "").parse::<f64>().unwrap_or(0.0)
}

fn fmt_money(raw: &str) -> String {
    let n = parse_amount(raw);
    let neg = n < 0.0;
    let abs = n.abs();
    let whole = abs.floor() as i64;
    let frac = ((abs - abs.floor()) * 100.0).round() as i64;
    let whole_s = whole
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(",");
    let s = format!("{whole_s}.{:02}", frac.max(0));
    if neg {
        format!("({s})")
    } else {
        s
    }
}

fn print_header(title: &str) {
    println!();
    println!("{}", "=".repeat(92));
    println!("{title}");
    println!("{}", "=".repeat(92));
}

/// Flat trial balance: Code | Account | Debit | Credit | Balance
fn print_trial_balance(body: &Value) {
    print_header("TRIAL BALANCE");
    println!(
        "{:<10} {:<40} {:>14} {:>14} {:>14}",
        "Code", "Account", "Debit", "Credit", "Balance"
    );
    println!("{}", "-".repeat(92));
    let mut td = 0.0;
    let mut tc = 0.0;
    for r in data_array(body) {
        let debit = s(r, "debit");
        let credit = s(r, "credit");
        td += parse_amount(&debit);
        tc += parse_amount(&credit);
        println!(
            "{:<10} {:<40} {:>14} {:>14} {:>14}",
            s(r, "code"),
            {
                let n = s(r, "name");
                n.chars().take(40).collect::<String>()
            },
            fmt_money(&debit),
            fmt_money(&credit),
            fmt_money(&s(r, "balance")),
        );
    }
    println!("{}", "-".repeat(92));
    println!(
        "{:<10} {:<40} {:>14} {:>14} {:>14}",
        "",
        "TOTALS",
        fmt_money(&td.to_string()),
        fmt_money(&tc.to_string()),
        fmt_money(&(td - tc).to_string()),
    );
    let diff = (td - tc).abs();
    if diff < 1.0 {
        println!("✓ Trial balance is in balance");
    } else {
        println!("✗ Trial balance OUT OF BALANCE by {}", fmt_money(&diff.to_string()));
    }
}

/// Tree statements (BS / IS / CF): indent by depth, amount on the right.
fn print_tree_node(node: &Value, depth: usize) {
    let code = s(node, "code");
    let name = s(node, "name");
    let total = s(node, "total");
    let amount = if total.is_empty() {
        s(node, "amount")
    } else {
        total
    };
    let indent = "  ".repeat(depth);
    let label = if code.is_empty() {
        name
    } else {
        format!("{code}  {name}")
    };
    let max_label = 74usize.saturating_sub(indent.len());
    let label: String = label.chars().take(max_label).collect();
    println!(
        "{indent}{label:<width$} {:>14}",
        fmt_money(&amount),
        width = max_label
    );
    if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
        for child in children {
            print_tree_node(child, depth + 1);
        }
    }
}

fn print_tree_statement(title: &str, body: &Value) {
    print_header(title);
    for node in data_array(body) {
        print_tree_node(node, 0);
        println!();
    }
}

/// Change of equity: components as rows, movements as columns (codes on margin)
fn print_equity_change(body: &Value) {
    print_header("STATEMENT OF CHANGES IN EQUITY");
    println!(
        "{:<10} {:<24} {:>14} {:>16} {:>14} {:>14}",
        "Code",
        "Component",
        "Opening",
        "Issues/contrib",
        "Profit",
        "Closing"
    );
    println!("{}", "-".repeat(100));
    for r in data_array(body) {
        println!(
            "{:<10} {:<24} {:>14} {:>16} {:>14} {:>14}",
            s(r, "code"),
            {
                let n = s(r, "name");
                n.chars().take(24).collect::<String>()
            },
            fmt_money(&s(r, "opening")),
            fmt_money(&s(r, "issues")),
            fmt_money(&s(r, "profit")),
            fmt_money(&s(r, "closing")),
        );
    }
    println!("{}", "-".repeat(100));
    println!(
        "(Each row: Opening + Issues/contrib + Profit = Closing. TOTAL row sums the components.)"
    );
}

fn render_report(path: &str, body: &Value) {
    if path.contains("trial-balance") {
        print_trial_balance(body);
    } else if path.contains("balancesheet") {
        print_tree_statement("STATEMENT OF FINANCIAL POSITION (Balance Sheet)", body);
    } else if path.contains("/income") {
        print_tree_statement("STATEMENT OF COMPREHENSIVE INCOME (P&L)", body);
    } else if path.contains("cashflow") {
        print_tree_statement("STATEMENT OF CASH FLOWS (Indirect)", body);
    } else if path.contains("change-of-equity") {
        print_equity_change(body);
    } else {
        print_header(path);
        println!("{body}");
    }
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
        render_report(path, &body);
        println!("✓ {path}");
    }

    println!(
        "✅ statements rendered for {} ({})",
        ctx.company_name, ctx.company_id
    );
    let _ = ctx.save_state();
    Ok(())
}
