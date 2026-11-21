//! RBAC grant/deny — second user with ProductionSupervisor can issue materials
//! but cannot process payroll.
mod common;

use common::{
    E2eResult, TestContext,
    client::{BASE, TestClient, extract_tokens, extract_uuid},
};
use serde_json::json;

fn ok(status: reqwest::StatusCode) -> bool {
    status.is_success() || status.as_u16() == 201
}

#[tokio::test]
async fn rbac_production_user_grant_deny() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Admin context: /api/rbac/me should show TenantAdmin
    let (s, me) = ctx
        .client
        .get("/api/rbac/me")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "admin /me: {s} {me}");
    let admin_roles = me["data"]["roles"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        admin_roles.iter().any(|r| r.as_str() == Some("TenantAdmin"))
            || me["data"]["is_tenant_admin"].as_bool() == Some(true),
        "creator should be tenant admin: {me}"
    );
    println!("✓ admin /api/rbac/me");

    // Register a second user (master only — no company yet)
    let email = format!("prod.user.{}@test.local", chrono::Utc::now().timestamp_millis());
    let http = reqwest::Client::new();
    let reg = http
        .post(format!("{BASE}/api/auth/register"))
        .json(&json!({
            "email": email,
            "password": "TestPass123!",
            "first_name": "Prod",
            "other_name": "User"
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let reg_status = reg.status();
    let reg_json: serde_json::Value = reg.json().await.map_err(|e| e.to_string())?;
    assert!(
        ok(reg_status),
        "register second user: {reg_status} {reg_json}"
    );
    let prod_user_uuid = extract_uuid(&reg_json, &["data", "user", "uuid"])
        .or_else(|_| extract_uuid(&reg_json, &["data", "uuid"]))
        .map_err(|e| e.to_string())?;
    println!("✓ second user {prod_user_uuid}");

    // Admin links them on master
    let (s, body) = ctx
        .client
        .post(
            "/api/companies/add-member",
            json!({ "user_uuid": prod_user_uuid }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "add-member: {s} {body}");
    println!("✓ linked to company");

    // Admin grants ProductionSupervisor + manufacturing module only (no payroll)
    let (s, body) = ctx
        .client
        .post(
            "/api/rbac/memberships",
            json!({
                "user_uuid": prod_user_uuid,
                "is_tenant_admin": false,
                "status": "active",
                "roles": ["ProductionSupervisor"],
                "modules": ["manufacturing", "inventory"]
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "upsert membership: {s} {body}");
    println!("✓ ProductionSupervisor membership");

    // Second user switches into company
    let (prod_access, prod_refresh) = extract_tokens(&reg_json).map_err(|e| e.to_string())?;
    let mut prod = TestClient::with_tokens(prod_access, prod_refresh);
    let (s, body) = prod
        .post(
            "/api/companies/switch-company",
            json!({ "company_id": ctx.company_id }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "switch-company: {s} {body}");
    let (access, refresh) = extract_tokens(&body).map_err(|e| e.to_string())?;
    prod.access = access;
    prod.refresh = refresh;
    println!("✓ prod user switched into company");

    let (s, me2) = prod
        .get("/api/rbac/me")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "prod /me: {s} {me2}");
    let perms = me2["data"]["permissions"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        perms
            .iter()
            .any(|p| p.as_str() == Some("manufacturing.issue")),
        "expected manufacturing.issue: {me2}"
    );
    assert!(
        !perms
            .iter()
            .any(|p| p.as_str() == Some("payroll.process")),
        "must not have payroll.process: {me2}"
    );
    println!("✓ prod permissions shaped correctly");

    // Deny: process payrun (even with a fake serial) → 403 Forbidden
    let (s, body) = prod
        .post("/api/payroll/payrun/process/1", json!({}))
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(
        s.as_u16(),
        403,
        "prod user must be forbidden from payroll process: {s} {body}"
    );
    println!("✓ payroll process denied (403)");

    // Allow path is harder without a live PO; assert issue endpoint is not 403
    // (may be 400/422/500 for missing payload — anything except 403 means authz passed)
    let (s, body) = prod
        .post(
            "/api/manufacturing/apply/direct/material",
            json!({
                "warehouse_serial_id": 1,
                "production_order_uuid": "00000000-0000-0000-0000-000000000001"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert_ne!(
        s.as_u16(),
        403,
        "prod user should not be forbidden from material issue: {s} {body}"
    );
    println!("✓ material issue not forbidden (status={})", s.as_u16());

    println!("✅ rbac_production_user_grant_deny OK");
    Ok(())
}
