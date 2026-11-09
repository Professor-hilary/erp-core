use std::collections::HashMap;

use sqlx::{PgPool, Row};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountRoleContext {
    pub role: AccountRole,
    pub context_key: String,
}

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
    PayrollLabourExpense,
    PayrollNetPayable,
    PayePayable,
    EmployeeNssfPayable,
    EmployerNssfPayable,
    EmployerNssfExpense,
    OtherPayrollDeductionsPayable,
    OtherEmployerPayrollExpense,
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
        Self::PayrollLabourExpense,
        Self::PayrollNetPayable,
        Self::PayePayable,
        Self::EmployeeNssfPayable,
        Self::EmployerNssfPayable,
        Self::EmployerNssfExpense,
        Self::OtherPayrollDeductionsPayable,
        Self::OtherEmployerPayrollExpense,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::AccountsReceivable => "accounts_receivable",
            Self::AccountsPayable => "accounts_payable",
            Self::SalesRevenue => "sales_revenue",
            Self::CostOfGoodsSold => "cost_of_goods_sold",
            Self::CashAtHand => "cash_at_hand",
            Self::CashAtBank => "cash_at_bank",
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
            Self::PayrollLabourExpense => "payroll_labour_expense",
            Self::PayrollNetPayable => "payroll_net_payable",
            Self::PayePayable => "paye_payable",
            Self::EmployeeNssfPayable => "employee_nssf_payable",
            Self::EmployerNssfPayable => "employer_nssf_payable",
            Self::EmployerNssfExpense => "employer_nssf_expense",
            Self::OtherPayrollDeductionsPayable => "other_payroll_deductions_payable",
            Self::OtherEmployerPayrollExpense => "other_employer_payroll_expense",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|role| role.key() == key)
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::AccountsReceivable => &["accounts receivable", "trade receivables", "debtors"],
            Self::AccountsPayable => &["accounts payable", "trade payables", "creditors"],
            Self::SalesRevenue => &["sales revenue", "sales", "sales of manufactured goods"],
            Self::CostOfGoodsSold => &["cost of goods sold", "cost of sales", "cogs"],
            Self::CashAtHand => &["cash on hand", "cash in hand", "cash at hand"],
            Self::CashAtBank => &["bank accounts", "bank", "cash at bank"],
            Self::RawMaterialsInventory => &["raw materials inventory", "raw materials"],
            Self::WorkInProgress => &["work in progress", "work in progress wip", "wip"],
            Self::FinishedGoodsInventory => &["finished goods inventory", "finished goods"],
            Self::MerchandiseInventory => &["merchandise inventory", "trading inventory"],
            Self::InventoryAdjustments => &["inventory adjustments", "inventory adjustment"],
            Self::InputVat => &["input vat", "vat input", "recoverable vat"],
            Self::OutputVat => &["output vat", "vat output", "sales tax payable"],
            Self::DirectMaterialsExpense => &["direct materials", "raw materials purchases"],
            Self::DirectLabourExpense => &["direct labour", "direct labor", "factory wages"],
            Self::WagesExpense => &["wages expense", "factory wages", "salaries and wages"],
            Self::WagesPayable => &["wages payable", "accrued wages", "salaries payable"],
            Self::ManufacturingOverheadControl => {
                &["manufacturing overhead control", "overhead variance control"]
            }
            Self::ManufacturingOverheadApplied => &["manufacturing overhead applied"],
            Self::MaterialUsageVariance => &["material usage variance"],
            Self::LabourEfficiencyVariance => {
                &["labour efficiency variance", "labor efficiency variance"]
            }
            Self::OverheadVariance => &["overhead variance"],
            Self::FixedAssetsCost => &["machinery equipment", "property plant equipment"],
            Self::AccumulatedDepreciation => &["accumulated depreciation"],
            Self::DepreciationExpense => &["depreciation expense", "depreciation office"],
            Self::ShareCapital => &["share capital", "owners capital", "owner capital"],
            Self::RetainedEarnings => &["retained earnings"],
            Self::DividendsPayable => &["dividends payable"],
            Self::PayrollLabourExpense => &["factory wages", "payroll expense"],
            Self::PayrollNetPayable => &["accrued wages", "payroll payable", "net wages payable"],
            Self::PayePayable => &["taxes payable", "paye payable", "income tax payable"],
            Self::EmployeeNssfPayable => &["social security", "nssf payable"],
            Self::EmployerNssfPayable => &["social security", "employer nssf payable"],
            Self::EmployerNssfExpense => &["payroll taxes factory", "payroll taxes - factory"],
            Self::OtherPayrollDeductionsPayable => &["payroll taxes payable", "accrued expenses"],
            Self::OtherEmployerPayrollExpense => &["labor benefits", "labour benefits"],
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
            | Self::DividendsPayable
            | Self::PayrollNetPayable
            | Self::PayePayable
            | Self::EmployeeNssfPayable
            | Self::EmployerNssfPayable
            | Self::OtherPayrollDeductionsPayable => &["liability"],

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
            | Self::DepreciationExpense
            | Self::PayrollLabourExpense
            | Self::EmployerNssfExpense
            | Self::OtherEmployerPayrollExpense => &["expense"],

            Self::ShareCapital | Self::RetainedEarnings => &["equity"],
        }
    }

    fn requires_context(self) -> bool {
        matches!(self, Self::CashAtHand | Self::CashAtBank)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedAccount {
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct AccountRoleResolver {
    accounts: HashMap<AccountRoleContext, ResolvedAccount>,
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
}

fn normalize_name(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

impl AccountRoleResolver {
    pub async fn seed_from_template_roles(
        pool: &PgPool,
        entries: &[(String, Vec<String>)],
    ) -> Result<MappingDiscoveryReport, AccountRoleError> {
        let mut report = MappingDiscoveryReport::default();
        for (code, role_keys) in entries {
            if role_keys.is_empty() {
                continue;
            }
            let account_uuid: Option<Uuid> = sqlx::query_scalar(
                r#"
                SELECT uuid FROM accounting.accounts
                WHERE code = $1 AND is_active = TRUE AND is_postable = TRUE
                "#,
            )
            .bind(code)
            .fetch_optional(pool)
            .await?;

            let Some(account_uuid) = account_uuid else {
                for key in role_keys {
                    report.missing.push(format!("{key}@{code}"));
                }
                continue;
            };

            for key in role_keys {
                let Some(role) = AccountRole::from_key(key) else {
                    report.missing.push(key.clone());
                    continue;
                };
                let result = sqlx::query(
                    r#"
                    INSERT INTO accounting.account_role_mappings (
                        role_key, context_key, account_uuid, mapping_source, is_active
                    )
                    VALUES ($1, 'default', $2, 'seed', TRUE)
                    ON CONFLICT (role_key, context_key) DO NOTHING
                    "#,
                )
                .bind(role.key())
                .bind(account_uuid)
                .execute(pool)
                .await?;
                if result.rows_affected() == 1 {
                    report.inserted += 1;
                    tracing::info!(role = role.key(), account_code = %code, "Seeded account role from CoA template");
                }
            }
        }
        Ok(report)
    }

    pub async fn discover_and_seed_defaults(
        pool: &PgPool,
    ) -> Result<MappingDiscoveryReport, AccountRoleError> {
        let rows = sqlx::query(
            r#"
            SELECT uuid, code, name, category FROM accounting.accounts
            WHERE is_active = TRUE AND is_postable = TRUE ORDER BY code
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
            });
        }

        let mut report = MappingDiscoveryReport::default();
        for role in AccountRole::ALL.iter().copied() {
            let existing: Option<Uuid> = sqlx::query_scalar(
                r#"
                SELECT account_uuid FROM accounting.account_role_mappings
                WHERE role_key = $1 AND context_key = 'default' AND is_active
                "#,
            )
            .bind(role.key())
            .fetch_optional(pool)
            .await?;
            if existing.is_some() {
                continue;
            }
            if role.requires_context() {
                continue;
            }

            let aliases: Vec<String> = role.aliases().iter().map(|n| normalize_name(n)).collect();
            let matches: Vec<&AccountCandidate> = candidates
                .iter()
                .filter(|a| role.allowed_categories().contains(&a.category.as_str()))
                .filter(|a| {
                    let name = normalize_name(&a.name);
                    aliases.iter().any(|alias| alias == &name)
                })
                .collect();

            match matches.as_slice() {
                [account] => {
                    let result = sqlx::query(
                        r#"
                        INSERT INTO accounting.account_role_mappings (
                            role_key, context_key, account_uuid, mapping_source, is_active
                        )
                        VALUES ($1, 'default', $2, 'discovery', TRUE)
                        ON CONFLICT (role_key, context_key) DO NOTHING
                        "#,
                    )
                    .bind(role.key())
                    .bind(account.uuid)
                    .execute(pool)
                    .await?;
                    if result.rows_affected() == 1 {
                        report.inserted += 1;
                    }
                }
                [] => report.missing.push(role.key().to_string()),
                _ => report.ambiguous.push(role.key().to_string()),
            }
        }
        Ok(report)
    }

    pub async fn load(pool: &PgPool) -> Result<Self, AccountRoleError> {
        let rows = sqlx::query(
            r#"
            SELECT m.role_key, m.context_key, a.uuid AS account_uuid, a.category
            FROM accounting.account_role_mappings m
            JOIN accounting.accounts a ON a.uuid = m.account_uuid
            WHERE m.is_active = TRUE AND a.is_active = TRUE AND a.is_postable = TRUE
            "#,
        )
        .fetch_all(pool)
        .await?;

        let mut accounts = HashMap::new();
        for row in rows {
            let role_key: String = row.try_get("role_key")?;
            let context_key: String = row.try_get("context_key")?;
            let account_uuid: Uuid = row.try_get("account_uuid")?;
            let category: String = row.try_get("category")?;
            let Some(role) = AccountRole::from_key(&role_key) else {
                return Err(AccountRoleError::UnknownRole(role_key));
            };
            if !role.allowed_categories().contains(&category.as_str()) {
                return Err(AccountRoleError::InvalidMapping(format!(
                    "role '{role_key}' mapped to incompatible category '{category}'"
                )));
            }
            accounts.insert(
                AccountRoleContext { role, context_key },
                ResolvedAccount { uuid: account_uuid },
            );
        }
        Ok(Self { accounts })
    }

    pub fn resolve(
        &self,
        role: AccountRole,
        context: &str,
    ) -> Result<&ResolvedAccount, AccountRoleError> {
        self.accounts
            .get(&AccountRoleContext {
                role,
                context_key: context.to_string(),
            })
            .ok_or_else(|| AccountRoleError::MissingMapping(role.key().to_string()))
    }

    pub fn resolve_default(&self, role: AccountRole) -> Result<&ResolvedAccount, AccountRoleError> {
        self.resolve(role, "default")
    }
}
