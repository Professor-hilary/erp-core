//! Deep smoke of currently wired read/write APIs on one manufacturing company.
//! Run after e2e_01_onboarding (and ideally e2e_02_setup_stakeholders).

mod common;

use common::{E2eResult, TestContext};
use serde_json::Value;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

fn data_array<'a>(v: &'a Value) -> Option<&'a Vec<Value>> {
    v.get("data").and_then(|d| d.as_array())
}

#[tokio::test]
async fn deep_api_surface_manufacturing_company() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;
    let client = &ctx.client;

    // ----- Companies -----
    let (status, body) = client
        .get("/api/companies/list")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "companies/list: {status} {body}");
    println!("✓ companies/list");

    // ----- COA -----
    let (status, body) = client
        .get("/api/accounts/list")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "accounts/list: {status} {body}");
    let accounts = data_array(&body).cloned().unwrap_or_default();
    assert!(!accounts.is_empty(), "COA empty");
    println!("✓ accounts/list ({})", accounts.len());

    // sample get-by-uuid if present
    if let Some(id) = accounts[0].get("uuid").and_then(|id| id.as_str()) {
        let (status, body) = client
            .get(&format!("/api/accounts/get/uuid/{id}"))
            .await
            .map_err(|error| error.to_string())?;
        assert!(ok(status), "accounts/get: {status} {body}");
        println!("✓ accounts/get/uuid");
    }

    // ----- Periods -----
    let (status, body) = client
        .get("/api/accounts/period/list")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "period/list: {status} {body}");
    let periods = data_array(&body).cloned().unwrap_or_default();
    assert!(!periods.is_empty(), "no financial periods");
    let open = periods
        .iter()
        .any(|period| period["is_open"].as_bool() == Some(true));
    assert!(open, "expected an open period after onboarding");
    println!("✓ period/list (open present)");

    // ----- Currencies (list is enough) -----
    let (status, body) = client
        .get("/api/accounts/currencies")
        .await
        .map_err(|error| error.to_string())?;
    // may be empty on new tenant — still must not 500
    assert!(
        ok(status) || status.as_u16() == 404,
        "currencies: {status} {body}"
    );
    println!("✓ accounts/currencies ({status})");

    // ----- Journal list (after capital/opening may be non-empty) -----
    let (status, body) = client
        .get("/api/transactions/list/brief")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "transactions/list/brief: {status} {body}");
    println!(
        "✓ transactions/list/brief (n={})",
        data_array(&body).map(|child| child.len()).unwrap_or(0)
    );

    let (status, body) = client
        .get("/api/transactions/list")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "transactions/list: {status} {body}");
    println!("✓ transactions/list");

    // ----- Ledger for cash if we know the uuid -----
    if let Some(cash) = ctx
        .account_uuid("110100")
        .or_else(|| ctx.account_uuid("110200"))
    {
        let (status, body) = client
            .get(&format!(
                "/api/transactions/get/ledger/{cash}?from=2025-01-01&to=2025-12-31&posted=true"
            ))
            .await
            .map_err(|error| error.to_string())?;
        assert!(ok(status), "ledger: {status} {body}");
        println!("✓ transactions/get/ledger/{{cash}}");
    }

    // ----- Workforce lists -----
    for path in [
        "/api/workforce/list/employees",
        "/api/workforce/list/departments",
        "/api/workforce/list/jobtitles",
    ] {
        let (status, body) = client.get(path).await.map_err(|error| error.to_string())?;
        assert!(ok(status), "{path}: {status} {body}");
        println!(
            "✓ {path} (n={})",
            data_array(&body).map(|child| child.len()).unwrap_or(0)
        );
    }

    // ----- Capital parties -----
    let (status, body) = client
        .get("/api/capital/party")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "capital/party list: {status} {body}");
    println!(
        "✓ capital/party (n={})",
        data_array(&body).map(|child| child.len()).unwrap_or(0)
    );

    for path in [
        "/api/capital/share-classes",
        "/api/capital/shareholders",
        "/api/capital/instruments",
        "/api/capital/facilities",
        "/api/capital/projects",
    ] {
        let (status, body) = client.get(path).await.map_err(|error| error.to_string())?;
        assert!(ok(status), "{path}: {status} {body}");
        println!("✓ {path}");
    }

    // ----- Procurement / sales lists -----
    let (status, body) = client
        .get("/api/procurement/list/vendors")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "list/vendors: {status} {body}");
    println!(
        "✓ procurement/list/vendors (n={})",
        data_array(&body).map(|child| child.len()).unwrap_or(0)
    );

    let (status, body) = client
        .get("/api/sales/list/customers")
        .await
        .map_err(|error| error.to_string())?;
    assert!(ok(status), "list/customers: {status} {body}");
    println!(
        "✓ sales/list/customers (n={})",
        data_array(&body).map(|child| child.len()).unwrap_or(0)
    );

    // ----- Inventory lists -----
    for path in [
        "/api/inventory/list/inventory",
        "/api/inventory/list/categories",
        "/api/inventory/list/warehouses",
    ] {
        let (status, body) = client.get(path).await.map_err(|error| error.to_string())?;
        assert!(ok(status), "{path}: {status} {body}");
        println!("✓ {path}");
    }

    // ----- Core reports (no extra path params) -----
    for path in [
        "/api/reports/trial-balance?as_of=2025-01-31",
        "/api/reports/balancesheet/single?as_of=2025-01-31",
        "/api/reports/income?from=2025-01-01&to=2025-01-31",
        "/api/reports/cashflow/indirect?from=2025-01-01&to=2025-01-31",
        "/api/reports/cashflow/direct?from=2025-01-01&to=2025-01-31",
        "/api/reports/change-of-equity?from=2025-01-01&to=2025-01-31",
        "/api/reports/balancesheet/compare?from=2025-01-01&to=2025-01-31",
    ] {
        let (status, body) = client.get(path).await.map_err(|error| error.to_string())?;
        assert!(ok(status), "report {path}: {status} {body}");
        println!("✓ {path}");
    }

    // ----- Negative: unauthenticated should fail -----
    let bare = reqwest::Client::new();
    let res = bare
        .get(format!("{}/api/accounts/list", common::BASE))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    assert!(
        res.status().as_u16() == 401 || res.status().as_u16() == 403,
        "expected 401/403 without token, got {}",
        res.status()
    );
    println!("✓ auth guard on /api/accounts/list");

    println!("✅ deep API surface OK for {}", ctx.company_name);
    Ok(())
}
