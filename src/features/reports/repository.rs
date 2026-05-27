use std::collections::HashMap;

// src/features/reports/repository.rs
use crate::{interface::api::errors::AppError, models::reports::*};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

//=====================================================================
// Helpers
//=====================================================================
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountBalance {
    pub code: String,
    pub name: String,
    pub category: String,
    pub cash_flow_category: Option<String>,
    pub normal_balance: String,
    pub is_contra: bool,
    pub end_balance: BigDecimal,
    pub begin_balance: BigDecimal,
}

fn prettify_activity_name(activity_type: &str) -> String {
    activity_type
        .split('_')
        .map(|s: &str| {
            let mut chars: std::str::Chars<'_> = s.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct ForceReload {
    pub force_reload: bool,
}

#[async_trait]
pub trait ReportRepository: Send + Sync {
    fn calculate_wc_adjustment(&self, acc: &AccountBalance, delta: &BigDecimal) -> BigDecimal;

    async fn get_balancesheet(
        &self,
        pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<Node>, AppError>;

    async fn get_balancesheet_comparison(
        &self,
        pool: &PgPool,
        as_of_1: NaiveDate,
        as_of_2: NaiveDate,
    ) -> Result<Vec<BalanceSheetCompareRow>, AppError>;

    async fn income_statement(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Node>, AppError>;

    async fn get_cf_direct(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Node>, AppError>;

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Node>, AppError>;

    async fn get_change_of_equity(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<EquityChangeRow>, AppError>;

    async fn get_aging_ar(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ArAgingDto>, AppError>;

    async fn get_aging_ap(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ApAgingDto>, AppError>;

    async fn get_sales_report(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError>;

    async fn get_purchase_report(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError>;

    async fn get_trial_balance(
        &self,
        pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<TrialBalanceRow>, AppError>;

    async fn get_inventory_valuation(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<InventoryValuationDto, AppError>;

    async fn get_customer_report(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<CustomerStatementDto, AppError>;

    async fn get_payroll(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<PayrollSummaryDto, AppError>;

    async fn refresh_reports(&self, pool: &PgPool, payload: &ForceReload) -> Result<(), AppError>;
}

pub struct PostgresReportRepo;

impl PostgresReportRepo {
    pub fn new() -> Self {
        Self
    }
}
#[async_trait]
impl ReportRepository for PostgresReportRepo {
    async fn get_balancesheet(
        &self,
        pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<Node>, AppError> {
        // 1) Pull flat balances from accounts table
        let rows: Vec<FlatAccount> = sqlx::query_as::<_, FlatAccount>(
            r#"SELECT
                acc.code,
                acc.name,
                acc.parent_code,
                acc.category,
                acc.normal_balance,
                acc.is_contra,
                COALESCE(SUM(te.debit) FILTER (WHERE te.created_at <= $1), 0)
                - COALESCE(SUM(te.credit) FILTER (WHERE te.created_at <= $1), 0)
                    AS balance
                FROM accounting.accounts acc
                LEFT JOIN accounting.transaction_entries te
                    ON te.account_uuid = acc.uuid
                WHERE acc.category IN ('asset', 'liability', 'equity')
                GROUP BY acc.code, acc.name, acc.parent_code,
                    acc.category, acc.normal_balance, acc.is_contra
            "#,
        )
        .bind(as_of)
        .fetch_all(pool)
        .await?;

        // 2) Normalize sign si:
        //  income  => positive when credit >  debit
        //  expense => positive when debit  > credit
        let mut nodes: HashMap<String, Node> = HashMap::new();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        for row in &rows {
            let mut signed: BigDecimal = row.balance.clone();

            // credit normal accounts (income) should be positive when credits exceed debits
            if row.normal_balance == "cr" {
                signed = -signed;
            }
            if row.is_contra {
                signed = -signed;
            }

            nodes.insert(
                row.code.clone(),
                Node {
                    code: row.code.clone(),
                    name: row.name.clone(),
                    category: row.category.clone(),
                    total: signed,
                    children: vec![],
                },
            );

            if let Some(p) = &row.parent_code {
                children_map
                    .entry(p.to_string())
                    .or_default()
                    .push(row.code.clone());
            }
        }

        // 3) Build + aggregate in one recursive closure
        fn build(
            code: &str,
            nodes: &mut HashMap<String, Node>,
            children_map: &HashMap<String, Vec<String>>,
        ) -> Option<Node> {
            let mut node = nodes.remove(code)?;

            if let Some(children) = children_map.get(code) {
                for c in children {
                    if let Some(child) = build(c, nodes, children_map) {
                        node.total += child.total.clone();
                        node.children.push(child);
                    }
                }
            }

            // 4) Prune zero branches
            if node.total.abs() < BigDecimal::from_f64(0.01).unwrap() && node.children.is_empty() {
                None
            } else {
                Some(node)
            }
        }

        // 5) Roots = accounts without parent
        let root_codes: Vec<String> = nodes
            .keys()
            .filter(|code| {
                !children_map
                    .values()
                    .any(|v: &Vec<String>| v.contains(code))
            })
            .cloned()
            .collect();

        // 6) Build final forest
        let mut result: Vec<Node> = Vec::new();
        for code in root_codes {
            if let Some(tree) = build(&code, &mut nodes, &children_map) {
                result.push(tree);
            }
        }

        // 7) Balance Sheet specific balances
        let mut total_assets = BigDecimal::from(0);
        let mut total_liabilities = BigDecimal::from(0);
        let mut total_equity = BigDecimal::from(0);

        for node in &result {
            match node.category.as_str() {
                "asset" => total_assets += node.total.clone(),
                "liability" => total_liabilities += node.total.clone(),
                "equity" => total_equity += node.total.clone(),
                _ => (),
            }
        }

        let total_liabilities_equity = total_liabilities.clone() + total_equity.clone();

        result.push(Node {
            code: "TOTAL_ASSETS".to_string(),
            name: "Total Assets".to_string(),
            category: "computed".to_string(),
            total: total_assets,
            children: vec![],
        });

        result.push(Node {
            code: "TOTAL_LIABILITIES".to_string(),
            name: "Total Liabilities".to_string(),
            category: "computed".to_string(),
            total: total_liabilities,
            children: vec![],
        });

        result.push(Node {
            code: "TOTAL_EQUITY".to_string(),
            name: "Total Equity".to_string(),
            category: "computed".to_string(),
            total: total_equity,
            children: vec![],
        });

        result.push(Node {
            code: "TOTAL_LIABILITIES_EQUITY".to_string(),
            name: "Total Liabilities + Equity".to_string(),
            category: "computed".to_string(),
            total: total_liabilities_equity,
            children: vec![],
        });

        Ok(result)
    }

    //=====================================================================
    // Repo Function
    //=====================================================================

    async fn get_cf_direct(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_close: NaiveDate,
    ) -> Result<Vec<Node>, AppError> {
        //=================================================================
        // 1. Fetch cash flow rows
        //=================================================================

        let rows: Vec<CashFlowRow> = sqlx::query_as::<_, CashFlowRow>(
            r#"
                SELECT
                    transaction_uuid,
                    transaction_entry_uuid,
                    activity_section,
                    activity_type,
                    direction,
                    amount,
                    description,
                    txn_date
                FROM accounting.cash_flow_entries
                WHERE txn_date >= $1
                AND txn_date < ($2 + INTERVAL '1 day')
                ORDER BY
                    activity_section,
                    activity_type,
                    txn_date,
                    transaction_uuid;
            "#,
        )
        .bind(period_start)
        .bind(period_close)
        .fetch_all(pool)
        .await?;

        //=====================================================================
        // DTOs
        //=====================================================================

        #[derive(Debug, Clone)]
        struct TransactionGroup {
            transaction_uuid: Uuid,
            description: String,
            total: BigDecimal,
            lines: Vec<CashFlowRow>,
        }

        #[derive(Debug, Clone)]
        struct ActivityGroup {
            section: String,
            activity_type: String,
            total: BigDecimal,
            transactions: Vec<TransactionGroup>,
        }

        #[derive(Debug, sqlx::FromRow)]
        pub struct CashBalanceRow {
            pub opening_cash: BigDecimal,
        }

        //=================================================================
        // 2. Hierarchical aggregation
        //
        // Structure:
        //
        // Section
        //   -> Activity Type
        //        -> Transactions
        //              -> Cash Lines
        //=================================================================

        let mut activity_groups: HashMap<(String, String), ActivityGroup> = HashMap::new();

        for row in rows {
            //=============================================================
            // Determine signed amount
            //=============================================================

            let signed_amount = if row.direction == "inflow" {
                row.amount.clone()
            } else {
                -row.amount.clone()
            };

            //=============================================================
            // Activity key
            //=============================================================

            let activity_key: (String, String) =
                (row.activity_section.clone(), row.activity_type.clone());

            //=============================================================
            // Get or create activity group
            //=============================================================

            let activity_group: &mut ActivityGroup =
                activity_groups
                    .entry(activity_key)
                    .or_insert(ActivityGroup {
                        section: row.activity_section.clone(),
                        activity_type: row.activity_type.clone(),
                        total: BigDecimal::from(0),
                        transactions: vec![],
                    });

            //=============================================================
            // Add to activity total
            //=============================================================

            activity_group.total += signed_amount.clone();

            //=============================================================
            // Locate transaction group
            //=============================================================

            if let Some(existing_txn) = activity_group
                .transactions
                .iter_mut()
                .find(|txn| txn.transaction_uuid == row.transaction_uuid)
            {
                existing_txn.total += signed_amount.clone();

                existing_txn.lines.push(row);
            } else {
                activity_group.transactions.push(TransactionGroup {
                    transaction_uuid: row.transaction_uuid,
                    description: row
                        .description
                        .clone()
                        .unwrap_or_else(|| "Transaction".to_string()),
                    total: signed_amount.clone(),
                    lines: vec![row],
                });
            }
        }

        //=================================================================
        // 3. Build section trees dynamically
        //=================================================================

        let mut section_nodes: HashMap<String, Vec<Node>> = HashMap::new();

        let mut section_totals: HashMap<String, BigDecimal> = HashMap::new();

        for (_, activity_group) in activity_groups {
            //=============================================================
            // Build transaction nodes
            //=============================================================

            let mut transaction_nodes: Vec<Node> = vec![];

            for txn in activity_group.transactions {
                //=========================================================
                // Optional line-level children
                //=========================================================

                let mut line_nodes: Vec<Node> = vec![];

                for line in txn.lines {
                    let signed_amount = if line.direction == "inflow" {
                        line.amount.clone()
                    } else {
                        -line.amount.clone()
                    };

                    line_nodes.push(Node {
                        code: line
                            .transaction_entry_uuid
                            .map(|id| id.to_string())
                            .unwrap_or_else(|| "line".to_string()),
                        name: line
                            .description
                            .clone()
                            .unwrap_or_else(|| "Cash Line".to_string()),
                        category: "cashflow_line".to_string(),
                        total: signed_amount,
                        children: vec![],
                    });
                }

                //=========================================================
                // Transaction node
                //=========================================================

                transaction_nodes.push(Node {
                    code: txn.transaction_uuid.to_string(),
                    name: txn.description,
                    category: "cashflow_transaction".to_string(),
                    total: txn.total,
                    children: line_nodes,
                });
            }

            //=============================================================
            // Activity node
            //=============================================================

            let activity_node: Node = Node {
                code: activity_group.activity_type.clone(),
                name: prettify_activity_name(&activity_group.activity_type),
                category: "cashflow_activity".to_string(),
                total: activity_group.total.clone(),
                children: transaction_nodes,
            };

            //=============================================================
            // Insert into section map
            //=============================================================

            section_nodes
                .entry(activity_group.section.clone())
                .or_insert(vec![])
                .push(activity_node);

            //=============================================================
            // Update section totals
            //=============================================================

            let entry: &mut BigDecimal = section_totals
                .entry(activity_group.section.clone())
                .or_insert(BigDecimal::from(0));

            *entry += activity_group.total.clone();
        }

        //=================================================================
        // 4. Build top-level section nodes
        //=================================================================

        let mut result: Vec<Node> = vec![];

        let mut net_cash_flow: BigDecimal = BigDecimal::from(0);

        let ordered_sections: Vec<(&str, &str)> = vec![
            ("operating", "Cash Flows from Operating Activities"),
            ("investing", "Cash Flows from Investing Activities"),
            ("financing", "Cash Flows from Financing Activities"),
        ];

        for (section_code, section_name) in ordered_sections {
            let children: Vec<Node> = section_nodes.remove(section_code).unwrap_or_default();

            let total: BigDecimal = section_totals
                .remove(section_code)
                .unwrap_or(BigDecimal::from(0));

            net_cash_flow += total.clone();

            result.push(Node {
                code: section_code.to_uppercase(),
                name: section_name.to_string(),
                category: "cashflow_section".to_string(),
                total,
                children,
            });
        }

        //=================================================================
        // 5. Opening cash
        //=================================================================

        let opening: CashBalanceRow = sqlx::query_as::<_, CashBalanceRow>(
            r#"
                SELECT
                    COALESCE(
                        SUM(te.debit - te.credit), 0
                    ) AS opening_cash
                FROM accounting.transaction_entries te
                JOIN accounting.accounts acc
                    ON acc.uuid = te.account_uuid
                WHERE acc.cash_flow_category = 'cash'
                AND te.created_at < $1
            "#,
        )
        .bind(period_start)
        .fetch_one(pool)
        .await?;

        //=================================================================
        // 6. Closing cash
        //=================================================================

        let closing_cash: BigDecimal = opening.opening_cash.clone() + net_cash_flow.clone();

        //=================================================================
        // 7. Computed nodes
        //=================================================================

        result.push(Node {
            code: "NET_CASH_FLOW".to_string(),
            name: "Net Increase / (Decrease) in Cash".to_string(),
            category: "computed".to_string(),
            total: net_cash_flow,
            children: vec![],
        });

        result.push(Node {
            code: "OPENING_CASH".to_string(),
            name: "Cash at Beginning of Period".to_string(),
            category: "computed".to_string(),
            total: opening.opening_cash,
            children: vec![],
        });

        result.push(Node {
            code: "CLOSING_CASH".to_string(),
            name: "Cash at End of Period".to_string(),
            category: "computed".to_string(),
            total: closing_cash,
            children: vec![],
        });

        Ok(result)
    }

    fn calculate_wc_adjustment(&self, acc: &AccountBalance, delta: &BigDecimal) -> BigDecimal {
        let mut adj: BigDecimal = delta.clone();

        // 1. Apply category logic
        match acc.category.as_str() {
            "asset" => {
                adj = -adj;
            }
            "liability" => {
                // Increase = cash inflow -> no flip
            }
            _ => return BigDecimal::zero(),
        }

        // 2. Increatse in current asset = cash outflow
        if acc.normal_balance == "cr" {
            adj = -adj.clone();
        }
        if acc.is_contra {
            adj = -adj.clone();
        }

        adj
    }

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Node>, AppError> {
        // 1. Get Net Profit from your existing income statement
        let income_nodes: Vec<Node> = self
            .income_statement(pool, period_start, period_end)
            .await?;
        let net_profit: BigDecimal = income_nodes
            .iter()
            .find(|n: &&Node| n.code == "NET_PROFIT")
            .map(|n: &Node| n.total.clone())
            .unwrap_or_else(BigDecimal::zero);

        // 2. Fetch all relevant account balances
        let accounts: Vec<AccountBalance> = sqlx::query_as::<_, AccountBalance>(
            r#"
        SELECT
            a.code,
            a.name,
            a.category,
            a.cash_flow_category,
            a.normal_balance,
            a.is_contra,
            -- Ending balance
            a.current_balance as end_balance,
            -- Beginning balance (from before period)
            COALESCE(
                (SELECT current_balance FROM accounting.accounts WHERE code = a.code),
                0
            ) -
            COALESCE(
                SUM(te.amount) FILTER (WHERE te.created_at BETWEEN $1 AND $2),
                0
            ) as begin_balance
        FROM accounting.accounts a
        LEFT JOIN accounting.transaction_entries te
            ON te.account_uuid = a.uuid
        WHERE a.is_active = true
          AND a.cash_flow_category IS NOT NULL
          AND a.cash_flow_category IN ('non-cash', 'working-capital')
          AND a.current_balance != 0
        GROUP BY a.code, a.name, a.category, a.cash_flow_category,
                 a.normal_balance, a.is_contra, a.current_balance
        "#,
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_all(pool)
        .await?;

        // 3. Calculate adjustments
        let mut non_cash_adjustments: BigDecimal = BigDecimal::zero();
        let mut working_capital_changes: BigDecimal = BigDecimal::zero();
        let mut wc_nodes: Vec<Node> = vec![];

        for acc in &accounts {
            let delta: BigDecimal = acc.end_balance.clone() - acc.begin_balance.clone();

            // Check that the account is not zero to proceed
            if delta.abs() < BigDecimal::from_f64(0.01).unwrap() {
                continue;
            }

            match acc.cash_flow_category.as_deref() {
                Some("non-cash") => {
                    // e.g., Depreciation, Amortization, Provisions
                    let signed: BigDecimal = if acc.normal_balance == "cr" {
                        delta.clone()
                    } else {
                        -delta.clone()
                    };
                    non_cash_adjustments += signed.clone();

                    // Optional: add as child node
                }
                Some("working-capital") => {
                    let adjustment: BigDecimal = self.calculate_wc_adjustment(acc, &delta);
                    working_capital_changes += adjustment.clone();

                    wc_nodes.push(Node {
                        code: acc.code.clone(),
                        name: acc.name.clone(),
                        category: "working_capital".to_string(),
                        total: adjustment,
                        children: vec![],
                    });
                }
                _ => {}
            }
        }

        let cash_from_operations: BigDecimal =
            net_profit.clone() + &non_cash_adjustments + working_capital_changes.clone();

        // 4. Build the tree (same structure as direct method)
        let mut result: Vec<Node> = vec![];

        // Operating Section (Indirect)
        let mut operating_children: Vec<Node> = vec![Node {
            code: "NET_PROFIT".to_string(),
            name: "Net Profit / (Loss)".to_string(),
            category: "cashflow_line".to_string(),
            total: net_profit,
            children: vec![],
        }];

        if !non_cash_adjustments.is_zero() {
            operating_children.push(Node {
                code: "NON_CASH_ADJ".to_string(),
                name: "Adjustments for non-cash items".to_string(),
                category: "cashflow_line".to_string(),
                total: non_cash_adjustments,
                children: vec![],
            });
        }

        if !wc_nodes.is_empty() {
            operating_children.push(Node {
                code: "WC_CHANGES".to_string(),
                name: "Changes in Working Capital".to_string(),
                category: "cashflow_activity".to_string(),
                total: working_capital_changes,
                children: wc_nodes,
            });
        }

        let operating_node: Node = Node {
            code: "OPERATING".to_string(),
            name: "Cash Flows from Operating Activities".to_string(),
            category: "cashflow_section".to_string(),
            total: cash_from_operations.clone(),
            children: operating_children,
        };
        result.push(operating_node);

        // 5. Investing & Financing — reuse direct method data
        let direct_cf: Vec<Node> = self.get_cf_direct(pool, period_start, period_end).await?;

        for node in direct_cf {
            if node.code == "INVESTING" || node.code == "FINANCING" {
                result.push(node);
            }
        }

        // 6. Net cash flow + opening/closing cash (same as direct)
        let net_cash_flow: BigDecimal = result.iter().map(|n| n.total.clone()).sum::<BigDecimal>();

        //=================================================================
        // 5. Opening cash
        //=================================================================

        let opening: CashBalanceRow = sqlx::query_as::<_, CashBalanceRow>(
            r#"
                SELECT
                    COALESCE(
                        SUM(te.debit - te.credit), 0
                    ) AS opening_cash
                FROM accounting.transaction_entries te
                JOIN accounting.accounts acc
                    ON acc.uuid = te.account_uuid
                WHERE acc.cash_flow_category = 'cash'
                AND te.created_at < $1
            "#,
        )
        .bind(period_start)
        .fetch_one(pool)
        .await?;

        //=================================================================
        // 6. Closing cash
        //=================================================================

        let closing_cash: BigDecimal = opening.opening_cash.clone() + net_cash_flow.clone();

        //=================================================================
        // 7. Computed nodes
        //=================================================================

        result.push(Node {
            code: "NET_CASH_FLOW".to_string(),
            name: "Net Increase / (Decrease) in Cash".to_string(),
            category: "computed".to_string(),
            total: net_cash_flow,
            children: vec![],
        });

        result.push(Node {
            code: "OPENING_CASH".to_string(),
            name: "Cash at Beginning of Period".to_string(),
            category: "computed".to_string(),
            total: opening.opening_cash,
            children: vec![],
        });

        result.push(Node {
            code: "CLOSING_CASH".to_string(),
            name: "Cash at End of Period".to_string(),
            category: "computed".to_string(),
            total: closing_cash,
            children: vec![],
        });

        Ok(result)
    }

    async fn get_balancesheet_comparison(
        &self,
        pool: &PgPool,
        as_of_1: NaiveDate,
        as_of_2: NaiveDate,
    ) -> Result<Vec<BalanceSheetCompareRow>, AppError> {
        let rows: Vec<BalanceSheetCompareRow> = sqlx::query_as::<_, BalanceSheetCompareRow>(
            r#"
        SELECT
            code,
            name,
            category,
            depth,
            path,
            balance_1,
            balance_2,
            delta
        FROM reporting.get_balance_sheet_compare($1, $2)
        ORDER BY path
        "#,
        )
        .bind(as_of_1)
        .bind(as_of_2)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn income_statement(
        &self,
        pool: &PgPool,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Node>, AppError> {
        // 1) Pull flat balances from accounts table
        let rows: Vec<FlatAccount> = sqlx::query_as::<_, FlatAccount>(
            r#"SELECT
                acc.code,
                acc.name,
                acc.parent_code,
                acc.category,
                acc.normal_balance,
                acc.is_contra,
                COALESCE(
                    SUM(te.debit) FILTER (WHERE te.created_at BETWEEN $1 AND $2), 0
                )
                - COALESCE(
                    SUM(te.credit) FILTER (WHERE te.created_at BETWEEN $1 AND $2), 0
                ) AS balance
                FROM accounting.accounts acc
                LEFT JOIN accounting.transaction_entries te
                    ON te.account_uuid = acc.uuid
                WHERE acc.category IN ('income', 'expense')
                GROUP BY acc.code, acc.name, acc.parent_code,
                    acc.category, acc.normal_balance, acc.is_contra
            "#,
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_all(pool)
        .await?;

        // 2) Normalize sign si:
        //  income  => positive when credit >  debit
        //  expense => positive when debit  > credit
        let mut nodes: HashMap<String, Node> = HashMap::new();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        for row in &rows {
            let mut signed: BigDecimal = row.balance.clone();

            // credit normal accounts (income) should be positive when credits exceed debits
            if row.normal_balance == "cr" {
                signed = -signed;
            }
            if row.is_contra {
                signed = -signed;
            }

            nodes.insert(
                row.code.clone(),
                Node {
                    code: row.code.clone(),
                    name: row.name.clone(),
                    category: row.category.clone(),
                    total: signed,
                    children: vec![],
                },
            );

            if let Some(p) = &row.parent_code {
                children_map
                    .entry(p.to_string())
                    .or_default()
                    .push(row.code.clone());
            }
        }

        // 3) Build + aggregate in one recursive closure
        fn build(
            code: &str,
            nodes: &mut HashMap<String, Node>,
            children_map: &HashMap<String, Vec<String>>,
        ) -> Option<Node> {
            let mut node = nodes.remove(code)?;

            if let Some(children) = children_map.get(code) {
                for c in children {
                    if let Some(child) = build(c, nodes, children_map) {
                        node.total += child.total.clone();
                        node.children.push(child);
                    }
                }
            }

            // 4) Prune zero branches
            if node.total.abs() < BigDecimal::from_f64(0.01).unwrap() && node.children.is_empty() {
                None
            } else {
                Some(node)
            }
        }

        // 5) Roots = accounts without parent
        let root_codes: Vec<String> = nodes
            .keys()
            .filter(|code: &&String| {
                !children_map
                    .values()
                    .any(|v: &Vec<String>| v.contains(code))
            })
            .cloned()
            .collect();

        // 6) Build final forest
        let mut result: Vec<Node> = Vec::new();
        for code in root_codes {
            if let Some(tree) = build(&code, &mut nodes, &children_map) {
                result.push(tree);
            }
        }

        // 7) Extract key sections
        let mut revenue: BigDecimal = BigDecimal::zero();
        let mut cogs: BigDecimal = BigDecimal::zero();
        let mut expenses: BigDecimal = BigDecimal::zero();

        for node in &result {
            let name: String = node.name.to_lowercase();

            if name.contains("revenue") || name.contains("income") {
                revenue += node.total.clone();
            } else if name.contains("cost of goods") || name.contains("cogs") {
                cogs += node.total.clone();
            } else if node.category == "expense" {
                expenses += node.total.clone();
            }
        }

        // 8) Compute profits
        let gross_profit: BigDecimal = revenue - cogs.clone();
        let operating_profits: BigDecimal = gross_profit.clone() - (expenses.clone() - cogs); //remove double-counting
        let net_profit: BigDecimal = gross_profit.clone() - expenses; // TODO: less other expenses add other incomes here

        // 9) Push computed nodes
        result.push(Node {
            code: "GROSS_PROFIT".to_string(),
            name: "Gross Profit".to_string(),
            category: "computed".to_string(),
            total: gross_profit,
            children: vec![],
        });

        result.push(Node {
            code: "OPERATING_PROFIT".to_string(),
            name: "Operating Profit".to_string(),
            category: "computed".to_string(),
            total: operating_profits,
            children: vec![],
        });

        result.push(Node {
            code: "NET_PROFIT".to_string(),
            name: "Net Profit".to_string(),
            category: "computed".to_string(),
            total: net_profit,
            children: vec![],
        });
        Ok(result)
    }

    async fn get_aging_ar(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<ArAgingDto>, AppError> {
        let rows = sqlx::query_as::<_, ArAgingDto>(
            r#"SELECT * FROM reporting.ar_aging_detailed WHERE customer_serial_id = (
                    SELECT serial_id FROM sales.customers WHERE uuid = $1
                )"#,
        )
        .bind(customer_uuid)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    async fn get_aging_ap(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<ApAgingDto>, AppError> {
        let rows = sqlx::query_as::<_, ApAgingDto>(
            r#"SELECT * FROM reporting.ap_aging_detailed WHERE vendor_serial_id = (
                    SELECT serial_id FROM procurement.vendors WHERE uuid = $1
                )"#,
        )
        .bind(vendor_uuid)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    async fn get_change_of_equity(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<EquityChangeRow>, AppError> {
        let rows = sqlx::query_as::<_, EquityChangeRow>(
            r#"
                SELECT code, name, description, amount
                FROM reporting.get_statement_of_changes_in_equity($1, $2)
                ORDER BY code, description
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn get_sales_report(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
        _user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError> {
        // We'll map customer_uuid -> account_uuid or filter by txn_date
        let from_d = from.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to_d = to.unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());

        let rows = sqlx::query_as::<_, CashbookRowDto>(
            r#"
            SELECT cb.*
            FROM reporting.cashbook cb
            JOIN sales.turnover i ON i.serial_id = cb.txn_serial_id  -- adapt as needed
            WHERE i.customer_uuid = $1
              AND cb.txn_date BETWEEN $2 AND $3
            ORDER BY cb.txn_date DESC
            "#,
        )
        .bind(customer_uuid)
        .bind(from_d)
        .bind(to_d)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn get_purchase_report(
        &self,
        pool: &PgPool,
        vendor_uuid: Uuid,
        _user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError> {
        let from_d = from.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to_d = to.unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());

        let rows = sqlx::query_as::<_, CashbookRowDto>(
            r#"
            SELECT cb.*
            FROM reporting.cashbook cb
            JOIN procurement.purchases b ON b.serial_id = cb.txn_serial_id  -- adapt as needed
            WHERE b.vendor_uuid = $1
              AND cb.txn_date BETWEEN $2 AND $3
            ORDER BY cb.txn_date DESC
            "#,
        )
        .bind(vendor_uuid)
        .bind(from_d)
        .bind(to_d)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    async fn get_trial_balance(
        &self,
        pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<TrialBalanceRow>, AppError> {
        let rows = sqlx::query_as::<_, TrialBalanceRow>(
            r#"SELECT * FROM reporting.get_trial_balance($1)"#,
        )
        .bind(as_of)
        .fetch_all(pool)
        .await?;

        // let total_debit : BigDecimal = rows.iter().map(|r| r.debit).sum();
        // let total_credit: BigDecimal = rows.iter().map(|r| r.credit).sum();
        // assert!((total_debit - total_credit).abs() < BigDecimal::zero());

        Ok(rows)
    }

    async fn get_inventory_valuation(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<InventoryValuationDto, AppError> {
        let row = sqlx::query_as::<_, InventoryValuationDto>(
            r#"SELECT * FROM reporting.inventory_valuation"#,
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    async fn get_customer_report(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<CustomerStatementDto, AppError> {
        let row = sqlx::query_as::<_, CustomerStatementDto>(
            r#"SELECT * FROM reporting.customer_statement"#,
        )
        .fetch_one(pool)
        .await?;
        Ok(row)
    }

    async fn get_payroll(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
    ) -> Result<PayrollSummaryDto, AppError> {
        let row =
            sqlx::query_as::<_, PayrollSummaryDto>(r#"SELECT * FROM reporting.payroll_summary"#)
                .fetch_one(pool)
                .await?;
        Ok(row)
    }

    async fn refresh_reports(&self, pool: &PgPool, payload: &ForceReload) -> Result<(), AppError> {
        sqlx::query(r#"SELECT reporting.refresh_all($1)"#)
            .bind(payload.force_reload)
            .execute(pool)
            .await?;

        Ok(())
    }
}
