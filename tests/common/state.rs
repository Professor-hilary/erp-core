use std::{collections::HashMap, error::Error, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::{TestClient, TestContext};

const STATE_FILE: &str = "target/e2e_state.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct E2eState {
    pub access: String,
    pub refresh: String,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub industry: String,
    pub accounts: HashMap<String, Uuid>,
    pub employees: HashMap<String, Uuid>,
    pub customers: HashMap<String, Uuid>,
    pub vendors: HashMap<String, Uuid>,
}

impl E2eState {
    pub fn path() -> PathBuf {
        PathBuf::from(STATE_FILE)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(dir) = Self::path().parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let raw = std::fs::read_to_string(Self::path())
            .with_context(|| format!("Missing {}. Run e2e_01_onboarding first.", STATE_FILE))?;
        Ok(serde_json::from_str(&raw)?)
    }
}

impl TestContext {
    /// Persist after onboarding (or after any phase that mutes maps)
    pub fn save_state(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        E2eState {
            access: self.client.access.clone(),
            refresh: self.client.refresh.clone(),
            user_id: self.user_id,
            company_id: self.company_id,
            company_name: self.company_name.clone(),
            industry: self.industry.clone(),
            accounts: self.accounts.clone(),
            employees: self.employees.clone(),
            customers: self.customers.clone(),
            vendors: self.vendors.clone(),
        }
        .save()
    }

    /// Continure the same company from a previous phase.
    pub async fn from_saved_state() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let state = E2eState::load()?;
        Ok(TestContext {
            client: TestClient::with_tokens(state.access, state.refresh),
            user_id: state.user_id,
            company_id: state.company_id,
            company_name: state.company_name,
            industry: state.industry,
            accounts: state.accounts,
            employees: state.employees,
            customers: state.customers,
            vendors: state.vendors,
        })
    }
}
