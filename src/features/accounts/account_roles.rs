use std::collections::HashMap;

use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountRole {
    AccountsReceivable,
    AccountsPayable,

    SalesRevenue,
    CostOfGoodsSold,

    CashAtHand,
    CashAtBank,

    RawMaterialsInventory,
    WorkInProgress,
    FinishedGoodsInventory,
    MerchandiseInventory,
    InventoryAdjustments,

    InputVat,
    OutputVat,

    DirectMaterialsExpense,
    DirectLabourExpense,
    WagesExpense,
    WagesPayable,

    ManufacturingOverheadControl,
    ManufacturingOverheadApplied,
    MaterialUsageVariance,
    LabourEfficiencyVariance,
    OverheadVariance,

    FixedAssetsCost,
    AccumulatedDepreciation,
    DepreciationExpense,

    ShareCapital,
    RetainedEarnings,
    DividendsPayable,
}

impl AccountRole {
    pub const ALL: &'static [Self] = &[
        Self::AccountsReceivable,
        Self::AccountsPayable,
        Self::SalesRevenue,
        Self::CostOfGoodsSold,
        Self::CashAtHand,
        Self::CashAtBank,
        Self::RawMaterialsInventory,
        Self::WorkInProgress,
        Self::FinishedGoodsInventory,
        Self::MerchandiseInventory,
        Self::InventoryAdjustments,
        Self::InputVat,
        Self::OutputVat,
        Self::DirectMaterialsExpense,
        Self::DirectLabourExpense,
        Self::WagesExpense,
        Self::WagesPayable,
        Self::ManufacturingOverheadControl,
        Self::ManufacturingOverheadApplied,
        Self::MaterialUsageVariance,
        Self::LabourEfficiencyVariance,
        Self::OverheadVariance,
        Self::FixedAssetsCost,
        Self::AccumulatedDepreciation,
        Self::DepreciationExpense,
        Self::ShareCapital,
        Self::RetainedEarnings,
        Self::DividendsPayable,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::AccountsReceivable => "accounts_receivable",
            Self::AccountsPayable => "accounts_payable",
            Self::SalesRevenue => "sales_revenue",
            Self::CostOfGoodsSold => "cost_of_goods_sold",
            Self::CashAtHand => "cash_on_hand",
            Self::CashAtBank => "bank",
            Self::RawMaterialsInventory => "raw_materials_inventory",
            Self::WorkInProgress => "work_in_progress",
            Self::FinishedGoodsInventory => "finished_goods_inventory",
            Self::MerchandiseInventory => "merchandise_inventory",
            Self::InventoryAdjustments => "inventory_adjustments",
            Self::InputVat => "input_vat",
            Self::OutputVat => "output_vat",
            Self::DirectMaterialsExpense => "direct_materials_expense",
            Self::DirectLabourExpense => "direct_labour_expense",
            Self::WagesExpense => "wages_expense",
            Self::WagesPayable => "wages_payable",
            Self::ManufacturingOverheadControl => "manufacturing_overhead_control",
            Self::ManufacturingOverheadApplied => "manufacturing_overhead_applied",
            Self::MaterialUsageVariance => "material_usage_variance",
            Self::LabourEfficiencyVariance => "labour_efficiency_variance",
            Self::OverheadVariance => "overhead_variance",
            Self::FixedAssetsCost => "fixed_assets_cost",
            Self::AccumulatedDepreciation => "accumulated_depreciation",
            Self::DepreciationExpense => "depreciation_expense",
            Self::ShareCapital => "share_capital",
            Self::RetainedEarnings => "retained_earnings",
            Self::DividendsPayable => "dividends_payable",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|role| role.key() == key)
    }

    /// Candidate names used by the automatic discovery process.
    ///
    /// These are suggestions, not account codes or UUIDs.
    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::AccountsReceivable => &["accounts receivable", "trade receivables", "debtors"],
            Self::AccountsPayable => &["accounts payable", "trade payables", "creditors"],
            Self::SalesRevenue => &["sales revenue", "sales", "revenue from sales"],
            Self::CostOfGoodsSold => &["cost of goods sold", "cost of sales"],
            Self::CashAtHand => &["cash on hand", "cash in hand", "cash at hand"],
            Self::CashAtBank => &["bank", "bank accounts"],
            Self::RawMaterialsInventory => &[
                "raw materials inventory",
                "raw material inventory",
                "raw materials",
            ],
            Self::WorkInProgress => &["work in progress", "work in progress wip", "wip"],
            Self::FinishedGoodsInventory => &["finished goods inventory", "finished goods"],
            Self::MerchandiseInventory => &["merchandise inventory", "trading inventory"],
            Self::InventoryAdjustments => &["inventory adjustments", "inventory adjustment"],
            Self::InputVat => &["input vat", "vat input", "recoverable vat"],
            Self::OutputVat => &["output vat", "vat output", "vat payable"],
            Self::DirectMaterialsExpense => {
                &["direct materials expense", "direct material expense"]
            }
            Self::DirectLabourExpense => &[
                "direct labour expense",
                "direct labor expense",
                "direct labour",
                "direct labor",
            ],
            Self::WagesExpense => &["wages expense", "salary expense", "salaries and wages"],
            Self::WagesPayable => &["wages payable", "salaries payable"],
            Self::ManufacturingOverheadControl => {
                &["manufacturing overhead control", "factory overhead control"]
            }
            Self::ManufacturingOverheadApplied => {
                &["manufacturing overhead applied", "applied factory overhead"]
            }
            Self::MaterialUsageVariance => &["material usage variance", "materials usage variance"],
            Self::LabourEfficiencyVariance => {
                &["labour efficiency variance", "labor efficiency variance"]
            }
            Self::OverheadVariance => &["overhead variance", "manufacturing overhead variance"],
            Self::FixedAssetsCost => &[
                "fixed assets",
                "property plant and equipment",
                "property plant equipment",
            ],
            Self::AccumulatedDepreciation => &["accumulated depreciation"],
            Self::DepreciationExpense => &["depreciation expense"],
            Self::ShareCapital => &["share capital", "ordinary share capital"],
            Self::RetainedEarnings => &["retained earnings", "retained profits"],
            Self::DividendsPayable => &["dividends payable", "dividend payable"],
        }
    }

    fn allowed_categories(self) -> &'static [&'static str] {
        match self {
            Self::AccountsReceivable
            | Self::CashAtHand
            | Self::CashAtBank
            | Self::RawMaterialsInventory
            | Self::WorkInProgress
            | Self::FinishedGoodsInventory
            | Self::MerchandiseInventory
            | Self::InputVat
            | Self::FixedAssetsCost
            | Self::AccumulatedDepreciation => &["asset"],

            Self::AccountsPayable
            | Self::OutputVat
            | Self::WagesPayable
            | Self::DividendsPayable => &["liability"],

            Self::SalesRevenue => &["income"],

            Self::CostOfGoodsSold
            | Self::InventoryAdjustments
            | Self::DirectMaterialsExpense
            | Self::DirectLabourExpense
            | Self::WagesExpense
            | Self::ManufacturingOverheadControl
            | Self::ManufacturingOverheadApplied
            | Self::MaterialUsageVariance
            | Self::LabourEfficiencyVariance
            | Self::OverheadVariance
            | Self::DepreciationExpense => &["expense"],

            Self::ShareCapital | Self::RetainedEarnings => &["equity"],
        }
    }

    /// Cash and bank accounts may be operational choices.
    /// Do not guess a universal default when multiple accounts may exist.
    fn requires_context(self) -> bool {
        match self(Self::CashAtHand => true, Self::CashAtBank => true, _=>false,)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedAccount {
    pub uuid: Uuid,
    pub code: String,
    pub name: String,
    pub category: String,
    pub normal_balance: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountRoleResolver {
    accounts: HashMap<AccountRole, ResolvedAccount>,
}

#[derive(Debug, Error)]
pub enum AccountRoleError {
    #[error("Database error while resolving account roles: {0}")]
    Database(#[from] sqlx::Error),

    #[error("No account is configured for role '{0}'")]
    MissingMapping(String),

    #[error("Invalid account-role mapping: {0}")]
    InvalidMapping(String),

    #[error("Unknown persisted account role '{0}'")]
    UnknownRole(String),
}

#[derive(Debug, Clone, Default)]
pub struct MappingDiscoveryReport {
    pub inserted: usize,
    pub ambiguous: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone)]
struct AccountCandidate {
    uuid: Uuid,
    code: String,
    name: String,
    category: String,
    normal_balance: String,
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

impl AccountRoleResolver {
    /// Discover safe defaults from the existing tenant CoA.
    ///
    /// Only a single, eligible, exact-name candidate is mapped.
    /// Existing manual mappings are never overwritten.
    pub async fn discover_and_seed_defaults(
        pool: &PgPool,
    ) -> Result<MappingDiscoveryReport, AccountRoleError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, code, name, category, normal_balance
            FROM accounting.accounts
            WHERE is_active = TRUE
              AND is_postable = TRUE
            ORDER BY code
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut candidates = Vec::with_capacity(rows.len());

        for row in rows {
            candidates.push(AccountCandidate {
                uuid: row.try_get("uuid")?,
                code: row.try_get("code")?,
                name: row.try_get("name")?,
                category: row.try_get("category")?,
                normal_balance: row.try_get("normal_balance")?,
            });
        }

        let mut report = MappingDiscoveryReport::default();

        for role in AccountRole::ALL.iter().copied() {
            if role.requires_context() {
                continue;
            }

            let aliases: Vec<String> = role
                .aliases()
                .iter()
                .map(|name| normalize_name(name))
                .collect();

            let matches: Vec<&AccountCandidate> = candidates
                .iter()
                .filter(|account| {
                    role.allowed_categories()
                        .contains(&account.category.as_str())
                })
                .filter(|account| {
                    let name = normalize_name(&account.name);
                    aliases.iter().any(|alias| alias == &name)
                })
                .collect();

            match matches.as_slice() {
                [account] => {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO accounting.account_role_mappings (
                            role_key,
                            account_uuid,
                            mapping_source,
                            is_active
                        )
                        VALUES ($1, $2, 'auto_discovered', TRUE)
                        ON CONFLICT (role_key) DO NOTHING
                        "#,
                    )
                    .bind(role.key())
                    .bind(account.uuid)
                    .execute(pool)
                    .await?;

                    if result.rows_affected() == 1 {
                        report.inserted += 1;

                        tracing::info!(
                            role = role.key(),
                            account_code = %account.code,
                            account_name = %account.name,
                            "Automatically configured account role"
                        );
                    }
                }

                [] => {
                    report.missing.push(role.key().to_string());
                }

                _ => {
                    report.ambiguous.push(role.key().to_string());

                    tracing::warn!(
                        role = role.key(),
                        candidates = matches.len(),
                        "Account-role discovery skipped an ambiguous mapping"
                    );
                }
            }
        }

        Ok(report)
    }

    /// Load validated mappings into the in-memory HashMap.
    pub async fn load(pool: &PgPool) -> Result<Self, AccountRoleError> {
        let rows = sqlx::query(
            r#"
            SELECT
                m.role_key,
                a.uuid AS account_uuid,
                a.code,
                a.name,
                a.category,
                a.normal_balance,
                a.is_active,
                a.is_postable
            FROM accounting.account_role_mappings AS m
            JOIN accounting.accounts AS a
              ON a.uuid = m.account_uuid
            WHERE m.is_active = TRUE
            ORDER BY m.role_key
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut accounts = HashMap::with_capacity(rows.len());

        for row in rows {
            let key: String = row.try_get("role_key")?;

            let role = AccountRole::from_key(&key)
                .ok_or_else(|| AccountRoleError::UnknownRole(key.clone()))?;

            let uuid: Uuid = row.try_get("account_uuid")?;
            let code: String = row.try_get("code")?;
            let name: String = row.try_get("name")?;
            let category: String = row.try_get("category")?;
            let normal_balance: String = row.try_get("normal_balance")?;
            let is_active: bool = row.try_get("is_active")?;
            let is_postable: bool = row.try_get("is_postable")?;

            if !is_active {
                return Err(AccountRoleError::InvalidMapping(format!(
                    "'{}' maps to inactive account '{}'",
                    key, code
                )));
            }

            if !is_postable {
                return Err(AccountRoleError::InvalidMapping(format!(
                    "'{}' maps to non-posting account '{}'",
                    key, code
                )));
            }

            if !role.allowed_categories().contains(&category.as_str()) {
                return Err(AccountRoleError::InvalidMapping(format!(
                    "'{}' maps to account '{}' with incompatible category '{}'",
                    key, code, category
                )));
            }

            accounts.insert(
                role,
                ResolvedAccount {
                    uuid,
                    code,
                    name,
                    category,
                    normal_balance,
                },
            );
        }

        Ok(Self { accounts })
    }

    pub fn resolve(&self, role: AccountRole) -> Result<&ResolvedAccount, AccountRoleError> {
        self.accounts
            .get(&role)
            .ok_or_else(|| AccountRoleError::MissingMapping(role.key().to_string()))
    }
}
