use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChartOfAccountsEntry {
    pub code: String,
    pub name: String,
    pub category: String,
    pub parent_code: Option<String>,
    pub normal_balance: String,
    pub is_contra: bool,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct CoaTemplate {
    // We must declare them so deserialization works
    pub industry: String,
    pub name: String,
    pub description: String,
    pub accounts: Vec<ChartOfAccountsEntry>,
}
