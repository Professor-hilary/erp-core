use crate::common::fixtures::{self, Person};

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use reqwest::{Response, StatusCode};
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

use super::client::{BASE, TestClient, extract_tokens, extract_uuid};

/// Shared state for one realistic company lifecycle.
/// Add fields as new modules come online (employees, items, etc.).
#[derive(Debug)]
#[allow(dead_code)]
pub struct TestContext {
    pub client: TestClient,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub industry: String,
    pub accounts: HashMap<String, Uuid>,
    pub employees: HashMap<String, Uuid>,
    pub customers: HashMap<String, Uuid>,
    pub vendors: HashMap<String, Uuid>,
    pub currencies: HashMap<String, Uuid>,
}

#[allow(dead_code)]
impl TestContext {
    // =========================================================================
    // 1. bootstrap_sme
    // =========================================================================
    /// Register unique user -> create company (tenant + COA + first period).
    pub async fn bootstrap_sme(industry: &str) -> Result<Self> {
        let stamp = chrono::Utc::now().timestamp_millis();
        let email = format!("sme_{stamp}@test.local");
        let password = "TestPass123!";
        let company_name = format!("Test Co {stamp}");

        // Register
        let http = reqwest::Client::new();
        let reg_body = json!({
            "email": email,
            "password": password,
            "first_name": "Test",
            "other_name": "Owner"
        });
        let reg_res: Response = http
            .post(format!("{BASE}/api/auth/register"))
            .json(&reg_body)
            .send()
            .await?;
        let reg_status: StatusCode = reg_res.status();
        let reg_json: Value = reg_res.json().await?;
        if !reg_status.is_success() && reg_status.as_u16() != 201 {
            println!("Failed auth");
            bail!("register failed ({reg_status}): {reg_json}");
        }
        let (access, refresh) = extract_tokens(&reg_json)?;
        let mut client: TestClient = TestClient::with_tokens(access, refresh);

        let user_id = extract_uuid(&reg_json, &["data", "user", "uuid"])
            .or_else(|_| extract_uuid(&reg_json, &["data", "uuid"]))
            .unwrap_or(Uuid::nil());

        // Create company
        let company_body: Value = json!({
            "name": company_name,
            "industry": industry,
            "country": "UG",
            "currency": "UGX",
            "business_type": "manufacturing",
            "period_type": "Yearly",
            "period_start": "2026-01-01"
        });
        let (status, company_json) = client.post("/api/companies/create", company_body).await?;
        if !status.is_success() && status.as_u16() != 201 {
            bail!("create company failed ({status}): {company_json}");
        }

        let (access, refresh) = extract_tokens(&company_json)?;
        client.access = access;
        client.refresh = refresh;

        let company_id: Uuid = extract_uuid(&company_json, &["data", "uuid"])?;
        let accounts: HashMap<String, Uuid> = Self::load_account_map(&client).await?;

        Ok(TestContext {
            client,
            user_id,
            company_id,
            company_name,
            industry: industry.to_string(),
            accounts,
            employees: HashMap::new(),
            customers: HashMap::new(),
            vendors: HashMap::new(),
            currencies: HashMap::new(),
        })
    }

    // =========================================================================
    // 2. raise_capital
    // =========================================================================
    /// Simple equity contribution: Dr Cash / Cr Owner's Equity.
    pub async fn raise_capital(&mut self, amount: &str) -> Result<Uuid> {
        let cash = self
            .account_uuid("110100")
            .or_else(|| self.account_uuid("110200"))
            .context("Cash/Bank account missing from COA")?;
        let equity = self
            .account_uuid("310100")
            .context("Owner's Equity (310100) missing from COA")?;

        let date = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        self.post_journal(
            date,
            "Owner capital contribution",
            &[
                (cash, amount, "0", "Cash in"),
                (equity, "0", amount, "Capital"),
            ],
            true,
        )
        .await
    }

    // =========================================================================
    // 3. set_opening_balances
    // =========================================================================
    /// Multi-line opening journal (adjust codes to your industry COA if needed).
    pub async fn set_opening_balances(&mut self) -> Result<Uuid> {
        let cash = self
            .account_uuid("110100")
            .or_else(|| self.account_uuid("110200"))
            .context("Cash missing")?;
        let inventory = self
            .account_uuid("110701")
            .or_else(|| self.account_uuid("110700"))
            .context("Inventory missing")?;
        let fixtures = self
            .account_uuid("120101")
            .or_else(|| self.account_uuid("120100"))
            .context("PPE/Fixtures missing")?;
        let equity = self.account_uuid("310100").context("Equity missing")?;
        let ap = self
            .account_uuid("210100")
            .context("Accounts Payable missing")?;

        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        self.post_journal(
            date,
            "Opening balances",
            &[
                (cash, "2000000", "0", "Opening cash"),
                (inventory, "5000000", "0", "Opening inventory"),
                (fixtures, "3000000", "0", "Opening fixtures"),
                (equity, "0", "9500000", "Opening equity"),
                (ap, "0", "500000", "Opening payables"),
            ],
            true,
        )
        .await
    }

    // =========================================================================
    // 4. close_current_period  (matches the new close_period design)
    // =========================================================================
    /// Finds the open period and closes it (snapshot + zero nominals + lock).
    pub async fn close_current_period(&mut self) -> Result<()> {
        let (status, periods) = self.client.get("/api/accounts/period/list").await?;
        if !status.is_success() {
            bail!("list periods failed ({status}): {periods}");
        }

        let arr = periods["data"]
            .as_array()
            .context("periods data is not an array")?;

        let open = arr
            .iter()
            .find(|period| period["is_open"].as_bool() == Some(true))
            .context("no open financial period found")?;

        let period_id = open["uuid"]
            .as_str()
            .context("period uuid missing")?
            .parse::<Uuid>()?;

        let (status, body) = self
            .client
            .patch(
                &format!("/api/accounts/period/close/{period_id}"),
                json!({}),
            )
            .await?;

        if !status.is_success() {
            bail!("close period failed ({status}): {body}");
        }

        // Optional: refresh account map after close (nominals should now be ~0)
        self.accounts = Self::load_account_map(&self.client).await?;
        Ok(())
    }

    // =========================================================================
    // Helpers used by the lifecycle methods (and by future ones)
    // =========================================================================

    async fn load_account_map(client: &TestClient) -> Result<HashMap<String, Uuid>> {
        let (status, value) = client.get("/api/accounts/list").await?;
        if !status.is_success() {
            bail!("list accounts failed ({status}): {value}");
        }
        let arr = value["data"]
            .as_array()
            .context("accounts data is not an array")?;
        let mut map = HashMap::new();
        for a in arr {
            let code = a["code"].as_str().unwrap_or("").to_string();
            if let Ok(id) = extract_uuid(a, &["uuid"]) {
                map.insert(code, id);
            }
        }
        if map.is_empty() {
            bail!("COA seeding produced zero accounts");
        }
        Ok(map)
    }

    pub fn account_uuid(&self, code: &str) -> Option<Uuid> {
        self.accounts.get(code).copied()
    }

    /// Generic multi-line journal. lines = (account, debit, credit, memo)
    pub async fn post_journal(
        &self,
        date: NaiveDate,
        description: &str,
        lines: &[(Uuid, &str, &str, &str)],
        posted: bool,
    ) -> Result<Uuid> {
        let line_objs: Vec<Value> = lines
            .iter()
            .map(|(acc, dr, cr, memo)| {
                json!({
                    "account_uuid": acc,
                    "debit": dr,
                    "credit": cr,
                    "memo": memo
                })
            })
            .collect();

        let body = json!({
            "txn_date": date.to_string(),
            "reference": format!("TEST-{}", Uuid::new_v4()),
            "description": description,
            "module": "journal",
            "posted": posted,
            "lines": line_objs
        });

        let (status, value) = self.client.post("/api/transactions/create", body).await?;
        if !status.is_success() && status.as_u16() != 201 {
            bail!("create journal failed ({status}): {value}");
        }

        extract_uuid(&value, &["data", "header", "uuid"])
            .or_else(|_| extract_uuid(&value, &["data", "uuid"]))
    }

    /// Period operating costs for realistic cash movement (factory + admin).
    pub async fn post_operating_costs(&self) -> Result<()> {
        let bank = self
            .account_uuid("110200")
            .or_else(|| self.account_uuid("110100"))
            .context("Bank/Cash missing")?;
        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();

        let items: &[(&str, &str, &str)] = &[
            ("540700", "2500000", "Factory electricity & lighting"),
            ("540200", "800000", "Factory utilities"),
            ("540600", "1200000", "Factory maintenance & repairs"),
            ("540100", "1500000", "Factory rent"),
            ("620500", "450000", "Telephone & internet"),
            ("620200", "900000", "Office rent"),
            ("620400", "350000", "Office supplies"),
        ];

        for (code, amount, memo) in items {
            let Some(exp) = self.account_uuid(code) else {
                println!("⚠ skip opex {code} — not in COA");
                continue;
            };
            self.post_journal(
                date,
                memo,
                &[(exp, amount, "0", memo), (bank, "0", amount, memo)],
                true,
            )
            .await?;
            println!("✓ opex {code} {amount} — {memo}");
        }
        Ok(())
    }

    pub async fn assert_trial_balance_ok(&self, as_of: &str) -> Result<()> {
        let (status, body) = self
            .client
            .get(&format!("/api/reports/trial-balance?as_of={as_of}"))
            .await?;
        if !status.is_success() {
            bail!("trial balance failed ({status}): {body}");
        }
        Ok(())
    }

    pub async fn seed_workforce(&mut self) -> Result<()> {
        use crate::common::client::extract_uuid;

        // ----- Departments -----
        let mut dept_ids: Vec<Uuid> = Vec::new();
        for (name, desc) in [
            ("Production", "Manufacturing floor"),
            ("Sales", "Order to cash"),
            ("Finance", "Accounts and payroll"),
            ("Warehouse", "Inventory and logistics"),
            ("Administration", "HR and office"),
        ] {
            let (s, value) = self
                .client
                .post(
                    "/api/workforce/create/department",
                    json!({ "name": name, "description": desc }),
                )
                .await?;
            if s.is_success() || s.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    dept_ids.push(id);
                }
            }
        }

        // ----- Job titles -----
        let mut title_ids: Vec<Uuid> = Vec::new();
        for title in [
            "Managing Director",
            "Production Manager",
            "Sales Executive",
            "Accountant",
            "Warehouse Supervisor",
            "Machine Operator",
            "Quality Controller",
            "Driver",
            "Admin Officer",
            "Technician",
        ] {
            let (s, value) = self
                .client
                .post(
                    "/api/workforce/create/jobtitle",
                    json!({ "title": title, "description": title }),
                )
                .await?;
            if s.is_success() || s.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    title_ids.push(id);
                }
            }
        }

        let default_dept = dept_ids.first().copied();
        let default_title = title_ids.first().copied();

        // ----- Employees (Onboarding by default) -----
        for person in fixtures::PEOPLE
            .iter()
            .filter(|person| person.role == "employee")
        {
            let (status, value) = self
                .client
                .post(
                    "/api/workforce/create/employee",
                    json!({
                        "first_name": person.first_name,
                        "last_name": person.last_name,
                        "email": person.email,
                        "phone_number": person.phone,
                        "hire_date": "2025-01-15",
                        "department_uuid": default_dept,
                        "job_title_uuid": default_title,
                        "employment_type": "Full-time",
                        "pay_frequency": "Monthly"
                    }),
                )
                .await?;

            if !(status.is_success() || status.as_u16() == 201) {
                eprintln!(
                    "employee create failed for {}: {status} {value}",
                    person.key
                );
                continue;
            }

            let emp_id = match extract_uuid(&value, &["data", "uuid"]) {
                Ok(id) => id,
                Err(_) => continue,
            };
            self.employees.insert(person.key.to_string(), emp_id);

            // Payment profile (bank on payslip)
            let (pp_status, pp_body) = self
                .client
                .put(
                    &format!("/api/workforce/employees/{emp_id}/payment-profile"),
                    json!({
                        "bank_name": "Stanbic",
                        "bank_account_number": format!("ACC-{}", &emp_id.to_string()[..8]),
                        "payment_method": "Bank Transfer",
                        "currency_code": "UGX"
                    }),
                )
                .await?;

            if !(pp_status.is_success() || pp_status.as_u16() == 201) {
                anyhow::bail!(
                    "Creating payment profile for {} failed: {} {}",
                    person.key,
                    pp_status,
                    pp_body
                );
            }

            // Statutory profile — DOB required for NSSF age band checks in payroll
            let (st_status, st_body) = self
                .client
                .put(
                    &format!("/api/workforce/employees/{emp_id}/statutory-profile"),
                    json!({
                        "tax_residency": "RESIDENT",
                        "nssf_covered": true,
                        "nssf_exempt": false,
                        "date_of_birth": "1990-06-15",
                        "tax_id": format!("TIN-{}", &emp_id.to_string()[..8]),
                        "social_security_number": format!("NSSF-{}", &emp_id.to_string()[..8])
                    }),
                )
                .await?;
            if !(st_status.is_success() || st_status.as_u16() == 201) {
                anyhow::bail!(
                    "Creating statutory profile for {} failed: {} {}",
                    person.key,
                    st_status,
                    st_body
                );
            }

            // Active contract = salary source of truth
            let (c_status, c_body) = self
                .client
                .post(
                    "/api/workforce/contracts",
                    json!({
                        "employee_uuid": emp_id,
                        "contract_type": "Permanent",
                        "start_date": "2025-01-15",
                        "job_title_uuid": default_title,
                        "department_uuid": default_dept,
                        "basic_salary": "550000",
                        "pay_frequency": "Monthly",
                        "employment_type": "Full-time",
                        "status": "Active"
                    }),
                )
                .await?;

            if !(c_status.is_success() || c_status.as_u16() == 201) {
                anyhow::bail!(
                    "Creating contract for {} failed: {} {}",
                    person.key,
                    c_status,
                    c_body
                );
            }

            // Activate for payroll eligibility (skip full checklist in seed)
            // Prefer completing onboarding tasks if you want strict path:
            let (ts, tasks) = self
                .client
                .get(&format!(
                    "/api/workforce/employees/{emp_id}/onboarding-tasks"
                ))
                .await?;
            if ts.is_success() {
                if let Some(arr) = tasks["data"].as_array() {
                    for t in arr {
                        if let Some(tid) = t["uuid"].as_str() {
                            let _ = self
                                .client
                                .post(
                                    &format!("/api/workforce/onboarding-tasks/{tid}/complete"),
                                    json!({ "notes": "e2e seed" }),
                                )
                                .await;
                        }
                    }
                }
            }
            let (as_, ab) = self
                .client
                .post(
                    &format!("/api/workforce/employees/{emp_id}/complete-onboarding"),
                    json!({}),
                )
                .await?;
            if !(as_.is_success() || as_.as_u16() == 201) {
                // If required tasks/SQL differ, surface once — payroll needs Active
                eprintln!("complete-onboarding {}: {as_} {ab}", person.key);
            }
        }

        // Bulk employees (scale) — same full path as heroes: contract + tasks → Active
        let hero_emp = self.employees.len();
        for i in 1..=fixtures::SCALE_EMPLOYEES {
            if self.employees.len() >= fixtures::SCALE_EMPLOYEES {
                break;
            }
            let key = format!("bulk_emp_{i:03}");
            let (status, value) = self
                .client
                .post(
                    "/api/workforce/create/employee",
                    json!({
                        "first_name": format!("Worker{i}"),
                        "last_name": format!("Batch{i}"),
                        "email": format!("bulk.emp.{i:03}@test.local"),
                        "phone_number": format!("+25670{:07}", 100000 + i),
                        "hire_date": "2025-06-01",
                        "department_uuid": default_dept,
                        "job_title_uuid": default_title,
                        "employment_type": "Full-time",
                        "pay_frequency": "Monthly"
                    }),
                )
                .await?;
            if !(status.is_success() || status.as_u16() == 201) {
                eprintln!("bulk employee create failed {key}: {status} {value}");
                continue;
            }
            let Ok(emp_id) = extract_uuid(&value, &["data", "uuid"]) else {
                continue;
            };
            self.employees.insert(key.clone(), emp_id);
            self.complete_employee_setup(
                emp_id,
                &key,
                "2025-06-01",
                default_dept,
                default_title,
                format!("BULK-{i:04}"),
                "1992-03-15",
                format!("TIN-B{i:05}"),
                format!("NSSF-B{i:05}"),
            )
            .await?;
        }
        eprintln!(
            "workforce seed: hero_employees≈{hero_emp} total_employees={}",
            self.employees.len()
        );

        Ok(())
    }

    async fn complete_employee_setup(
        &self,
        employee_uuid: Uuid,
        key: &str,
        hire_date: &str,
        department_uuid: Option<Uuid>,
        job_title_uuid: Option<Uuid>,
        bank_account_number: String,
        date_of_birth: &str,
        tax_id: String,
        social_security_number: String,
    ) -> Result<()> {
        // 1. Payment profile
        let (status, body) = self
            .client
            .put(
                &format!("/api/workforce/employees/{employee_uuid}/payment-profile"),
                json!({
                    "bank_name": "Stanbic",
                    "bank_account_number": bank_account_number,
                    "payment_method": "Bank Transfer",
                    "currency_code": "UGX"
                }),
            )
            .await?;

        if !status.is_success() {
            anyhow::bail!("{key}: payment profile failed: {status} {body}");
        }

        // 2. Statutory profile
        let (status, body) = self
            .client
            .put(
                &format!("/api/workforce/employees/{employee_uuid}/statutory-profile"),
                json!({
                    "tax_residency": "RESIDENT",
                    "nssf_covered": true,
                    "nssf_exempt": false,
                    "date_of_birth": date_of_birth,
                    "tax_id": tax_id,
                    "social_security_number": social_security_number
                }),
            )
            .await?;

        if !status.is_success() {
            anyhow::bail!("{key}: statutory profile failed: {status} {body}");
        }

        // 3. Employment contract
        let (status, body) = self
            .client
            .post(
                "/api/workforce/contracts",
                json!({
                    "employee_uuid": employee_uuid,
                    "contract_type": "Permanent",
                    "start_date": hire_date,
                    "job_title_uuid": job_title_uuid,
                    "department_uuid": department_uuid,
                    "basic_salary": "550000",
                    "pay_frequency": "Monthly",
                    "employment_type": "Full-time",
                    "status": "Active"
                }),
            )
            .await?;

        if !status.is_success() {
            anyhow::bail!("{key}: employment contract failed: {status} {body}");
        }

        // 4. Retrieve onboarding tasks
        let (status, tasks) = self
            .client
            .get(&format!(
                "/api/workforce/employees/{employee_uuid}/onboarding-tasks"
            ))
            .await?;

        if !status.is_success() {
            anyhow::bail!("{key}: retrieving onboarding tasks failed: {status} {tasks}");
        }

        let task_list = tasks["data"].as_array().ok_or_else(|| {
            anyhow::anyhow!("{key}: onboarding response did not contain a task array")
        })?;

        // 5. Complete every onboarding task
        for task in task_list {
            let task_uuid = task["uuid"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("{key}: onboarding task has no UUID"))?;

            if task["is_done"].as_bool() == Some(true) {
                continue;
            }

            let (status, body) = self
                .client
                .post(
                    &format!("/api/workforce/onboarding-tasks/{task_uuid}/complete"),
                    json!({
                        "notes": "Completed during E2E employee setup"
                    }),
                )
                .await?;

            if !status.is_success() {
                anyhow::bail!(
                    "{key}: completing onboarding task {task_uuid} failed: \
	                {status} {body}"
                );
            }
        }

        // 6. Complete the employee lifecycle
        let (status, body) = self
            .client
            .post(
                &format!("/api/workforce/employees/{employee_uuid}/complete-onboarding"),
                json!({}),
            )
            .await?;

        if !status.is_success() {
            anyhow::bail!("{key}: employee activation failed: {status} {body}");
        }

        Ok(())
    }

    /// Capital parties: owners, shareholders, lenders (people + institutions).
    pub async fn seed_capital_parties(&mut self) -> Result<()> {
        // Individual parties
        for person in fixtures::PEOPLE.iter().filter(|person: &&Person| {
            matches!(
                person.role,
                "owner" | "shareholder" | "lender_contact" | "board"
            )
        }) {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "legal_name": format!("{} {}", person.first_name, person.last_name),
                        "short_name": person.first_name,
                        "party_type": match person.role {
                            "owner" | "shareholder" | "board" => "SHAREHOLDER",
                            "lender_contact" => "LENDER",
                            _ => "OTHER",
                        },
                        "role": person.role,
                        "country_code": "UG",
                        "is_related_party": matches!(person.role, "owner" | "board"),
                    }),
                )
                .await?;
        }

        // Institutional parties (banks, lenders, investors)
        for inst in fixtures::INSTITUTIONS
            .iter()
            .filter(|institute| matches!(institute.kind, "bank" | "lender" | "investor"))
        {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "legal_name": inst.name,
                        "short_name": inst.key,  // or a short form of the name
                        "party_type": match inst.kind {
                            "bank" | "lender" => "LENDER",
                            "investor" => "INVESTOR",
                            _ => "OTHER",
                        },
                        "registration_number": inst.tax_id,
                        "country_code": "UG",
                        "is_related_party": false,
                    }),
                )
                .await?;
        }

        // Bulk investors / shareholders / lenders for volume testing
        for i in 1..=fixtures::SCALE_INVESTORS {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "legal_name": format!("Bulk Investor {i:03} Ltd"),
                        "short_name": format!("INV-{i:03}"),
                        "party_type": "INVESTOR",
                        "role": "investor",
                        "country_code": "UG",
                        "is_related_party": false
                    }),
                )
                .await?;
        }
        for i in 1..=fixtures::SCALE_OWNERS_SHAREHOLDERS {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "legal_name": format!("Shareholder {i:03}"),
                        "short_name": format!("SH-{i:03}"),
                        "party_type": "SHAREHOLDER",
                        "role": "shareholder",
                        "country_code": "UG",
                        "is_related_party": i <= 3
                    }),
                )
                .await?;
        }
        for i in 1..=fixtures::SCALE_LENDERS {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "legal_name": format!("Bulk Lender Bank {i:02}"),
                        "short_name": format!("LEND-{i:02}"),
                        "party_type": "LENDER",
                        "role": "lender",
                        "country_code": "UG",
                        "is_related_party": false
                    }),
                )
                .await?;
        }

        Ok(())
    }

    /// Vendors (suppliers) + customers from institutions.
    pub async fn seed_trading_partners(&mut self) -> Result<()> {
        for inst in fixtures::INSTITUTIONS
            .iter()
            .filter(|institute| institute.kind == "supplier")
        {
            let (status, value) = self
                .client
                .post(
                    "/api/procurement/create/vendor", // adjust path to your real route
                    json!({
                        "name": inst.name,
                        "code": inst.key,           // e.g. "sup_agri"
                        "email": inst.email,
                        "phone": null,
                        "credit_limit": "10000000"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    self.vendors.insert(inst.key.to_string(), id);
                }
            }
        }

        for inst in fixtures::INSTITUTIONS
            .iter()
            .filter(|institute| institute.kind == "customer")
        {
            let (status, value) = self
                .client
                .post(
                    "/api/sales/create/customer", // adjust path
                    json!({
                        "name": inst.name,
                        "code": inst.key,           // e.g. "cust_freshmart"
                        "email": inst.email,
                        "credit_limit": "5000000"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    self.customers.insert(inst.key.to_string(), id);
                }
            }
        }

        for i in 1..=fixtures::SCALE_SUPPLIERS {
            let code = format!("sup_bulk_{i:03}");
            if self.vendors.contains_key(&code) {
                continue;
            }
            let (status, value) = self
                .client
                .post(
                    "/api/procurement/create/vendor",
                    json!({
                        "name": format!("Bulk Supplier {i:03}"),
                        "code": code,
                        "email": format!("sup.bulk.{i:03}@test.local"),
                        "phone": null,
                        "credit_limit": "25000000"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    self.vendors.insert(code, id);
                }
            }
        }
        for i in 1..=fixtures::SCALE_CUSTOMERS {
            let code = format!("cust_bulk_{i:03}");
            if self.customers.contains_key(&code) {
                continue;
            }
            let (status, value) = self
                .client
                .post(
                    "/api/sales/create/customer",
                    json!({
                        "name": format!("Bulk Customer {i:03}"),
                        "code": code,
                        "email": format!("cust.bulk.{i:03}@test.local"),
                        "credit_limit": "15000000"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "uuid"]) {
                    self.customers.insert(code, id);
                }
            }
        }
        eprintln!(
            "trading partners: vendors={} customers={}",
            self.vendors.len(),
            self.customers.len()
        );
        Ok(())
    }

    /// Functional + common FX currencies (idempotent-ish: skip codes already present).
    pub async fn seed_currencies(&mut self) -> Result<()> {
        // Load existing
        let (status, body) = self.client.get("/api/accounts/currencies").await?;
        if status.is_success() {
            if let Some(arr) = body["data"].as_array() {
                for c in arr {
                    let code = c["code"].as_str().unwrap_or("").to_string();
                    if let Ok(id) = extract_uuid(c, &["id"]) {
                        self.currencies.insert(code, id);
                    }
                }
            }
        }

        for (code, name, decimals) in [
            ("UGX", "Ugandan Shilling", 0),
            ("USD", "US Dollar", 2),
            ("EUR", "Euro", 2),
            ("KES", "Kenyan Shilling", 2),
        ] {
            if self.currencies.contains_key(code) {
                continue;
            }
            let (status, value) = self
                .client
                .post(
                    "/api/accounts/currencies",
                    json!({
                        "code": code,
                        "name": name,
                        "decimal_places": decimals,
                        "is_active": true
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&value, &["data", "id"]) {
                    self.currencies.insert(code.to_string(), id);
                }
            }
        }
        Ok(())
    }

    pub fn currency_id(&self, code: &str) -> Option<Uuid> {
        self.currencies.get(code).copied()
    }
}
