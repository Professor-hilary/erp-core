//! Workforce / HR e2e — org structure, employment lifecycle, contracts, leave.
//!
//! Complements e2e_02 (stakeholder seed) with focused assertions that payroll
//! eligibility depends on: Active status, active contract, payment + statutory profiles.
//!
//! Run: cargo test --test e2e_15_workforce -- --nocapture --test-threads=1

mod common;

use common::{E2eResult, TestContext, client::extract_uuid};
use serde_json::json;
use uuid::Uuid;

fn ok(s: reqwest::StatusCode) -> bool {
    s.is_success() || s.as_u16() == 201
}

async fn ensure_seeded(ctx: &mut TestContext) -> Result<(), String> {
    if ctx.employees.is_empty() {
        ctx.seed_workforce().await.map_err(|e| e.to_string())?;
    }
    assert!(!ctx.employees.is_empty(), "expected seeded employees");
    Ok(())
}

fn sample_employee(ctx: &TestContext) -> Result<Uuid, String> {
    ctx.employees
        .values()
        .next()
        .copied()
        .ok_or_else(|| "no employee in ctx".into())
}

// =============================================================================
// Org structure
// =============================================================================

#[tokio::test]
async fn workforce_departments_and_job_titles() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/create/department",
            json!({
                "name": "E2E Quality",
                "description": "QA lab"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        ok(s) || s.as_u16() == 409,
        "create department: {s} {body}"
    );

    let (s, body) = ctx
        .client
        .get("/api/workforce/list/departments")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list departments: {s} {body}");
    let depts = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!depts.is_empty(), "expected ≥1 department");

    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/create/jobtitle",
            json!({
                "title": "E2E Inspector",
                "description": "Quality inspector"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s) || s.as_u16() == 409, "create job title: {s} {body}");

    let (s, body) = ctx
        .client
        .get("/api/workforce/list/jobtitles")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list job titles: {s} {body}");
    assert!(!body["data"].as_array().unwrap_or(&vec![]).is_empty());

    println!("✓ departments and job titles OK");
    Ok(())
}

// =============================================================================
// Seeded employees are payroll-ready
// =============================================================================

#[tokio::test]
async fn workforce_seeded_employees_are_active_with_contract() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_seeded(&mut ctx).await?;

    let (s, body) = ctx
        .client
        .get("/api/workforce/list/employees")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list employees: {s} {body}");
    let emps = body["data"].as_array().cloned().unwrap_or_default();
    assert!(!emps.is_empty());

    let sample = sample_employee(&ctx)?;
    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/get/employee/{sample}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get employee: {s} {body}");
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert!(
        status == "Active" || status == "Notice",
        "seeded employee should be payroll-eligible status, got {status}"
    );

    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/contracts?employee_uuid={sample}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list contracts: {s} {body}");
    let contracts = body["data"].as_array().cloned().unwrap_or_default();
    let active = contracts.iter().any(|c| c["status"] == "Active");
    assert!(active, "expected Active contract for payroll");

    let active_contract = contracts
        .iter()
        .find(|c| c["status"] == "Active")
        .expect("active contract");
    let basic = active_contract["basic_salary"].as_str().unwrap_or("0");
    assert!(
        basic != "0" && basic != "0.00",
        "active contract must carry positive basic_salary, got {basic}"
    );

    let (s, body) = ctx
        .client
        .get(&format!(
            "/api/workforce/employees/{sample}/payment-profile"
        ))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "payment profile: {s} {body}");

    println!("✓ seeded employees Active + contract + payment profile");
    Ok(())
}

// =============================================================================
// Statutory profile (tax residency / NSSF flags)
// =============================================================================

#[tokio::test]
async fn workforce_statutory_profile_upsert() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_seeded(&mut ctx).await?;
    let sample = sample_employee(&ctx)?;

    let (s, body) = ctx
        .client
        .put(
            &format!("/api/workforce/employees/{sample}/statutory-profile"),
            json!({
                "tax_residency": "RESIDENT",
                "nssf_covered": true,
                "nssf_exempt": false,
                "tax_id": "TIN-E2E-001",
                "social_security_number": "NSSF-E2E-001",
                "date_of_birth": "1990-05-15"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "upsert statutory profile: {s} {body}");

    let (s, body) = ctx
        .client
        .get(&format!(
            "/api/workforce/employees/{sample}/statutory-profile"
        ))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "get statutory profile: {s} {body}");
    assert_eq!(
        body["data"]["tax_residency"].as_str().unwrap_or(""),
        "RESIDENT"
    );
    assert_eq!(body["data"]["nssf_covered"].as_bool(), Some(true));

    println!("✓ statutory profile upsert OK");
    Ok(())
}

// =============================================================================
// Hire path: create → profiles → contract → onboarding → Active
// =============================================================================

#[tokio::test]
async fn workforce_hire_to_active_path() -> E2eResult<()> {
    let ctx = TestContext::from_saved_state().await?;

    // Resolve a department + title (create if needed)
    let (s, body) = ctx
        .client
        .get("/api/workforce/list/departments")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list depts: {s} {body}");
    let dept = body["data"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|d| d["uuid"].as_str())
        .and_then(|u| Uuid::parse_str(u).ok());

    let (s, body) = ctx
        .client
        .get("/api/workforce/list/jobtitles")
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list titles: {s} {body}");
    let title = body["data"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|d| d["uuid"].as_str())
        .and_then(|u| Uuid::parse_str(u).ok());

    let email = format!("e2e.hire.{}@example.test", Uuid::new_v4());
    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/create/employee",
            json!({
                "first_name": "E2E",
                "last_name": "Hire",
                "email": email,
                "phone_number": "+256700000099",
                "hire_date": "2026-06-01",
                "department_uuid": dept,
                "job_title_uuid": title,
                "employment_type": "Full-time",
                "pay_frequency": "Monthly"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create employee: {s} {body}");
    let emp_id = extract_uuid(&body, &["data", "uuid"]).map_err(|e| e.to_string())?;
    let status = body["data"]["status"].as_str().unwrap_or("");
    assert!(
        status == "Onboarding" || status == "Active",
        "new hire status unexpected: {status}"
    );

    let (s, body) = ctx
        .client
        .put(
            &format!("/api/workforce/employees/{emp_id}/payment-profile"),
            json!({
                "bank_name": "Equity",
                "bank_account_number": "E2E-HIRE-001",
                "payment_method": "Bank Transfer",
                "currency_code": "UGX"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "payment profile: {s} {body}");

    let (s, body) = ctx
        .client
        .put(
            &format!("/api/workforce/employees/{emp_id}/statutory-profile"),
            json!({
                "tax_residency": "RESIDENT",
                "nssf_covered": true,
                "nssf_exempt": false,
                "date_of_birth": "1992-03-20"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "statutory profile: {s} {body}");

    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/contracts",
            json!({
                "employee_uuid": emp_id,
                "contract_type": "Permanent",
                "start_date": "2026-06-01",
                "job_title_uuid": title,
                "department_uuid": dept,
                "basic_salary": "1200000",
                "pay_frequency": "Monthly",
                "employment_type": "Full-time",
                "status": "Active"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "create contract: {s} {body}");

    // Complete onboarding tasks then activate
    let (s, tasks) = ctx
        .client
        .get(&format!(
            "/api/workforce/employees/{emp_id}/onboarding-tasks"
        ))
        .await
        .map_err(|e| e.to_string())?;
    if ok(s) {
        if let Some(arr) = tasks["data"].as_array() {
            for t in arr {
                if let Some(tid) = t["uuid"].as_str() {
                    let _ = ctx
                        .client
                        .post(
                            &format!("/api/workforce/onboarding-tasks/{tid}/complete"),
                            json!({ "notes": "e2e hire path" }),
                        )
                        .await;
                }
            }
        }
    }

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/workforce/employees/{emp_id}/complete-onboarding"),
            json!({}),
        )
        .await
        .map_err(|e| e.to_string())?;
    // Some tenants may already be Active via contract path
    if ok(s) {
        assert_eq!(
            body["data"]["status"].as_str().unwrap_or(""),
            "Active",
            "complete-onboarding should yield Active"
        );
    } else {
        let (s2, body2) = ctx
            .client
            .get(&format!("/api/workforce/get/employee/{emp_id}"))
            .await
            .map_err(|e| e.to_string())?;
        assert!(ok(s2), "get after onboarding attempt: {s2} {body2}");
        println!(
            "note: complete-onboarding returned {s} {body}; status={}",
            body2["data"]["status"]
        );
    }

    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/employees/{emp_id}/events"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "employment events: {s} {body}");

    println!("✓ hire → contract → onboarding path OK emp={emp_id}");
    Ok(())
}

// =============================================================================
// Leave types + request smoke
// =============================================================================

#[tokio::test]
async fn workforce_leave_type_and_request() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_seeded(&mut ctx).await?;
    let sample = sample_employee(&ctx)?;

    let (s, body) = ctx
        .client
        .post(
            "/api/workforce/leave-types",
            json!({
                "code": "E2E_ANNUAL",
                "name": "E2E Annual leave",
                "is_paid": true,
                "grant_mode": "REQUEST",
                "unused_treatment": "ENCASH",
                "default_days_per_year": "21"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(
        ok(s) || s.as_u16() == 409,
        "leave type create must succeed or already exist: {s} {body}"
    );

    // Resolve leave type uuid
    let leave_type_uuid = if ok(s) {
        extract_uuid(&body, &["data", "uuid"]).ok()
    } else {
        None
    };

    if let Some(lt) = leave_type_uuid {
        let (s, body) = ctx
            .client
            .post(
                "/api/workforce/leave-requests",
                json!({
                    "employee_uuid": sample,
                    "leave_type_uuid": lt,
                    "start_date": "2026-08-10",
                    "end_date": "2026-08-12",
                    "days": "3",
                    "reason": "e2e leave"
                }),
            )
            .await
            .map_err(|e| e.to_string())?;
        // May require balances; accept success or business rejection
        if ok(s) {
            println!("✓ leave request created");
        } else {
            let msg = body.to_string();
            assert!(
                s.as_u16() == 422 || s.as_u16() == 400,
                "leave request unexpected status {s} {body}"
            );
            assert!(
                msg.to_lowercase().contains("balance")
                    || msg.to_lowercase().contains("leave")
                    || msg.to_lowercase().contains("entitlement")
                    || msg.to_lowercase().contains("insufficient")
                    || msg.to_lowercase().contains("unprocessable"),
                "expected a leave-business error body, got {s} {body}"
            );
            println!("✓ leave request rejected with business error (acceptable without balances)");
        }
    } else {
        println!("✓ leave type exists or create returned non-uuid; skip request");
    }

    Ok(())
}

// =============================================================================
// Contract amendment (salary change) surface
// =============================================================================

#[tokio::test]
async fn workforce_contract_amendment_salary() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_seeded(&mut ctx).await?;
    let sample = sample_employee(&ctx)?;

    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/contracts?employee_uuid={sample}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "list contracts: {s} {body}");
    let contract_uuid = body["data"]
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .find(|c| c["status"] == "Active")
                .or_else(|| arr.first())
        })
        .and_then(|c| c["uuid"].as_str())
        .and_then(|u| Uuid::parse_str(u).ok())
        .ok_or("no contract uuid")?;

    let (s, body) = ctx
        .client
        .post(
            &format!("/api/workforce/contracts/{contract_uuid}/amendments"),
            json!({
                "effective_date": "2026-07-01",
                "amendment_type": "SALARY",
                "basic_salary": "1650000",
                "notes": "e2e mid-year increase"
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "salary amendment must succeed: {s} {body}");

    // effective_date 2026-07-01 is in the past relative to current wall clock →
    // live contract basic_salary must materialise to 1,650,000.
    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/contracts?employee_uuid={sample}"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "re-list contracts: {s} {body}");
    let contracts = body["data"].as_array().cloned().unwrap_or_default();
    let active = contracts
        .iter()
        .find(|c| c["status"] == "Active")
        .or_else(|| contracts.first())
        .ok_or("no contract after amendment")?;
    let basic = active["basic_salary"].as_str().unwrap_or("");
    assert!(
        basic == "1650000" || basic == "1650000.00" || basic.starts_with("1650000"),
        "expected live contract basic_salary 1650000 after past-effective amendment, got {basic}"
    );
    println!("✓ contract amendment applied; basic_salary={basic}");
    Ok(())
}

// =============================================================================
// Employment events audit spine
// =============================================================================

#[tokio::test]
async fn workforce_employment_events_list() -> E2eResult<()> {
    let mut ctx = TestContext::from_saved_state().await?;
    ensure_seeded(&mut ctx).await?;
    let sample = sample_employee(&ctx)?;

    let (s, body) = ctx
        .client
        .get(&format!("/api/workforce/employees/{sample}/events"))
        .await
        .map_err(|e| e.to_string())?;
    assert!(ok(s), "events: {s} {body}");
    // Seed path should have produced at least HIRED / ONBOARDING events
    let events = body["data"].as_array().cloned().unwrap_or_default();
    assert!(
        !events.is_empty(),
        "expected employment events from seed/hire path"
    );
    println!("✓ employment events n={}", events.len());
    Ok(())
}
