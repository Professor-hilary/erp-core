use crate::common::fixtures;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;

use super::client::{BASE, TestClient, extract_tokens, extract_uuid};

/// Shared state for one realistic company lifecycle.
/// Add fields as new modules come online (employees, items, etc.).
#[derive(Debug)]
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
}

impl TestContext {
    // =========================================================================
    // 1. bootstrap_sme
    // =========================================================================
    /// Register unique user → create company (tenant + COA + first period).
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
        let reg_res = http
            .post(format!("{BASE}/api/auth/register"))
            .json(&reg_body)
            .send()
            .await?;
        let reg_status = reg_res.status();
        let reg_json: Value = reg_res.json().await?;
        if !reg_status.is_success() && reg_status.as_u16() != 201 {
            bail!("register failed ({reg_status}): {reg_json}");
        }
        let (access, refresh) = extract_tokens(&reg_json)?;
        let mut client = TestClient::with_tokens(access, refresh);

        let user_id = extract_uuid(&reg_json, &["data", "user", "uuid"])
            .or_else(|_| extract_uuid(&reg_json, &["data", "uuid"]))
            .unwrap_or(Uuid::nil());

        // Create company
        let company_body = json!({
            "name": company_name,
            "industry": industry,
            "country": "UG",
            "currency": "UGX",
            "business_type": "services",
            "period_type": "yearly",
            "period_start": "2025-01-01"
        });
        let (status, company_json) = client.post("/api/companies/create", company_body).await?;
        if !status.is_success() && status.as_u16() != 201 {
            bail!("create company failed ({status}): {company_json}");
        }

        let (access, refresh) = extract_tokens(&company_json)?;
        client.access = access;
        client.refresh = refresh;

        let company_id = extract_uuid(&company_json, &["data", "uuid"])?;
        let accounts = Self::load_account_map(&client).await?;

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
            .account_uuid("310000")
            .context("Owner's Equity (310000) missing from COA")?;

        let date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
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
        let equity = self.account_uuid("310000").context("Equity missing")?;
        let ap = self
            .account_uuid("210100")
            .context("Accounts Payable missing")?;

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
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
            .find(|p| p["is_open"].as_bool() == Some(true))
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
        let (status, v) = client.get("/api/accounts/list").await?;
        if !status.is_success() {
            bail!("list accounts failed ({status}): {v}");
        }
        let arr = v["data"]
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

        let (status, v) = self.client.post("/api/transactions/create", body).await?;
        if !status.is_success() && status.as_u16() != 201 {
            bail!("create journal failed ({status}): {v}");
        }

        extract_uuid(&v, &["data", "header", "uuid"])
            .or_else(|_| extract_uuid(&v, &["data", "uuid"]))
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
        // Departments
        for (name, desc) in [
            ("Production", "Manufacturing floor"),
            ("Sales", "Order to cash"),
            ("Finance", "Accounts and payroll"),
            ("Warehouse", "Inventory and logistics"),
            ("Administration", "HR and office"),
        ] {
            let (_s, v) = self
                .client
                .post(
                    "/api/workforce/create/department",
                    json!({ "name": name, "description": desc }),
                )
                .await?;
            let _ = v; // store dept ids if your API returns them
        }

        // Job titles
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
            let _ = self
                .client
                .post(
                    "/api/workforce/create/jobtitle",
                    json!({ "title": title, "description": title }),
                )
                .await?;
        }

        // Employees (≥10 from PEOPLE with role employee)
        for p in fixtures::PEOPLE.iter().filter(|p| p.role == "employee") {
            let (status, v) = self
                .client
                .post(
                    "/api/workforce/create/employee",
                    json!({
                        "first_name": p.first_name,
                        "last_name": p.last_name,
                        "email": p.email,
                        "phone_number": p.phone,
                        "hire_date": "2025-01-15",
                        "employment_type": "full_time",
                        "salary": "1500000",
                        "pay_frequency": "monthly",
                        "status": "active"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&v, &["data", "uuid"]) {
                    self.employees.insert(p.key.to_string(), id);
                }
            }
        }
        Ok(())
    }

    /// Capital parties: owners, shareholders, lenders (people + institutions).
    pub async fn seed_capital_parties(&mut self) -> Result<()> {
        // Individual parties
        for p in fixtures::PEOPLE
            .iter()
            .filter(|p| matches!(p.role, "owner" | "shareholder" | "lender_contact" | "board"))
        {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "name": format!("{} {}", p.first_name, p.last_name),
                        "party_type": p.role,
                        "email": p.email,
                        "phone": p.phone,
                        "is_individual": true
                    }),
                )
                .await?;
        }

        // Institutional parties (banks, lenders, investors)
        for inst in fixtures::INSTITUTIONS
            .iter()
            .filter(|i| matches!(i.kind, "bank" | "lender" | "investor"))
        {
            let _ = self
                .client
                .post(
                    "/api/capital/party",
                    json!({
                        "name": inst.name,
                        "party_type": inst.kind,
                        "email": inst.email,
                        "tax_id": inst.tax_id,
                        "is_individual": false
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
            .filter(|i| i.kind == "supplier")
        {
            let (status, v) = self
                .client
                .post(
                    "/api/procurement/vendors", // adjust path to your real route
                    json!({
                        "name": inst.name,
                        "email": inst.email,
                        "tax_id": inst.tax_id,
                        "currency": "UGX"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&v, &["data", "uuid"]) {
                    self.vendors.insert(inst.key.to_string(), id);
                }
            }
        }

        for inst in fixtures::INSTITUTIONS
            .iter()
            .filter(|i| i.kind == "customer")
        {
            let (status, v) = self
                .client
                .post(
                    "/api/sales/customers", // adjust path
                    json!({
                        "name": inst.name,
                        "email": inst.email,
                        "tax_id": inst.tax_id,
                        "currency": "UGX"
                    }),
                )
                .await?;
            if status.is_success() || status.as_u16() == 201 {
                if let Ok(id) = extract_uuid(&v, &["data", "uuid"]) {
                    self.customers.insert(inst.key.to_string(), id);
                }
            }
        }
        Ok(())
    }
}
