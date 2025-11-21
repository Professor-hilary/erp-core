use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChartOfAccountsEntry {
    pub code: String,
    pub name: String,
    pub category: String,
    pub parent_code: String,
    pub normal_balance: String,
    pub is_contra: bool,
}
