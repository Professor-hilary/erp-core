use std::collections::HashMap;

// src/features/reports/repository.rs
use crate::{interface::api::errors::AppError, models::reports::*};

use async_trait::async_trait;
use bigdecimal::{BigDecimal, FromPrimitive, Zero};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct ForceReload {
    pub force_reload: bool,
}

#[async_trait]
pub trait ReportRepository: Send + Sync {
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
    ) -> Result<Vec<CashflowGroup>, AppError>;

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CashFlowRow>, AppError>;

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

        // 7) Extract key sections
        let mut revenue = BigDecimal::zero();
        let mut cogs = BigDecimal::zero();
        let mut expenses = BigDecimal::zero();

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
        let operating_profits: BigDecimal = gross_profit.clone() - (expenses - cogs); //remove double-counting
        let net_profit: BigDecimal = operating_profits.clone(); // TODO: less other expenses add other incomes here

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

    async fn get_cf_direct(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CashflowGroup>, AppError> {
        let _rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
            r#"
                SELECT activity_group, code, name, SUM(debit) AS inflow, SUM(credit) AS outflow
                FROM reporting.get_cashflow_direct_full($1, $2)
                GROUP BY activity_group, code, name
                ORDER BY activity_group, code
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await?;

        use std::collections::HashMap;

        let /*mut*/ groups_map: HashMap<String, Vec<CashflowItem>> = HashMap::new();

        // for row in rows {
        //     let group: String = row.get("activity_group");
        //     let item: CashflowItem = CashflowItem {
        //         code: row.get("code"),
        //         name: row.get("name"),
        //         inflow: row.get::<f64, _>("inflow"),
        //         outflow: row.get::<f64, _>("outflow"),
        //         net_cash: row.get::<f64, _>("inflow") - row.get::<f64, _>("outflow"),
        //     };
        //     groups_map.entry(group).or_default().push(item);
        // }

        let mut result = Vec::new();
        for (group_name, accounts) in groups_map {
            let total_inflow: f64 = accounts.iter().map(|a| a.inflow).sum();
            let total_outflow: f64 = accounts.iter().map(|a| a.outflow).sum();
            let total_net_cash: f64 = accounts.iter().map(|a| a.net_cash).sum();

            result.push(CashflowGroup {
                group_name,
                accounts,
                total_inflow,
                total_outflow,
                total_net_cash,
            });
        }

        Ok(result)
    }

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CashFlowRow>, AppError> {
        // Using a tuple because the SQL returns (section, description, amount)
        let rows = sqlx::query_as::<_, CashFlowRow>(
            "SELECT section, description, amount FROM reporting.get_cash_flow_statement($1, $2)",
        )
        .bind(start)
        .bind(end)
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
