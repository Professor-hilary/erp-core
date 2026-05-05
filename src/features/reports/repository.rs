use std::collections::HashMap;

// src/features/reports/repository.rs
use crate::{interface::api::errors::AppError, models::reports::*};

use async_trait::async_trait;
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
    ) -> Result<Vec<BalanceSheetRow>, AppError>;

    async fn get_balancesheet_comparison(
        &self,
        pool: &PgPool,
        as_of_1: NaiveDate,
        as_of_2: NaiveDate,
    ) -> Result<Vec<BalanceSheetCompareRow>, AppError>;

    // async fn get_income(
    //     &self,
    //     pool: &PgPool,
    //     start: NaiveDate,
    //     end: NaiveDate,
    // ) -> Result<Vec<IncomeStatementRow>, AppError>;

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
    ) -> Result<Vec<BalanceSheetRow>, AppError> {
        let rows = sqlx::query_as::<_, BalanceSheetRow>(
            r#"SELECT
                code,
                name,
                category,
                depth,
                path,
                balance
            FROM reporting.get_balance_sheet($1)
            ORDER BY path
            "#,
        )
        .bind(as_of)
        .fetch_all(pool)
        .await?;

        for row in &rows {
            let indent = "  ".repeat(row.depth as usize);
            println!("{}{}  {}", indent, row.name, row.balance);
        }

        Ok(rows)
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

    // async fn get_income(
    //     &self,
    //     pool: &PgPool,
    //     start: NaiveDate,
    //     end: NaiveDate,
    // ) -> Result<Vec<IncomeStatementRow>, AppError> {
    //     let rows = sqlx::query_as::<_, IncomeStatementRow>(
    //         r#"SELECT code, name, category, depth, path, balance
    //         FROM reporting.get_income_statement($1, $2)
    //         ORDER BY path"#,
    //     )
    //     .bind(start)
    //     .bind(end)
    //     .fetch_all(pool)
    //     .await?;

    //     println!("\n=== INCOME STATEMENT DEBUG ===\n");

    //     for row in &rows {
    //         let indent = "    ".repeat(row.depth as usize);

    //         println!(
    //             "{}{} [{}] {} → {}",
    //             indent,
    //             if row.depth == 0 { "►" } else { "↳" },
    //             row.code,
    //             row.name,
    //             row.balance
    //         );
    //     }

    //     println!("\n=== END DEBUG ===\n");

    //     Ok(rows)
    // }

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
            let mut signed = row.balance;

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
                        node.total += child.total;
                        node.children.push(child);
                    }
                }
            }

            // 4) Prune zero branches
            if node.total.abs() < 0.01 && node.children.is_empty() {
                None
            } else {
                Some(node)
            }
        }

        // 5) Roots = accounts without parent
        let root_codes: Vec<String> = nodes
            .keys()
            .filter(|code| !children_map.values().any(|v| v.contains(code)))
            .cloned()
            .collect();

        // 6) Build final forest
        let mut result = Vec::new();
        for code in root_codes {
            if let Some(tree) = build(&code, &mut nodes, &children_map) {
                result.push(tree);
            }
        }

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
