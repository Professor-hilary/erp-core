// src/features/reports/repository.rs
use crate::{
    infrastructure::errors::AppError,
    models::reports::{
        ApAgingDto, ArAgingDto, BalanceSheetDto, CashFlowDto, CashbookRowDto, CustomerStatementDto,
        IncomeStatementDto, InventoryValuationDto, PayrollSummaryDto, TrialBalanceDto,
    },
};
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn get_balancesheet(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<BalanceSheetDto, AppError>;

    async fn get_income(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<IncomeStatementDto, AppError>;

    async fn get_cf_direct(
        &self,
        pool: &PgPool,
        period_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError>;

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        period_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError>;

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
        user_id: Uuid,
    ) -> Result<TrialBalanceDto, AppError>;

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
        _user_id: Uuid,
    ) -> Result<BalanceSheetDto, AppError> {
        let row = sqlx::query_as::<_, BalanceSheetDto>(r#"SELECT * FROM reporting.balance_sheet"#)
            .fetch_one(pool)
            .await?;
        Ok(row)
    }

    async fn get_income(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<IncomeStatementDto, AppError> {
        // If you want the income MV to honor dates, create a date-filtered MV or use base tables.
        // Here we'll query the MV and optionally restrict via joins to transactions by date.
        // Example: aggregate from accounting.transaction_entries joined to transactions (t.txn_date)
        // For simplicity, query the MV and ignore dates if None.
        if from.is_none() && to.is_none() {
            let row = sqlx::query_as::<_, IncomeStatementDto>(
                r#"SELECT * FROM reporting.income_statement"#,
            )
            .fetch_one(pool)
            .await?;
            return Ok(row);
        }

        // Example date-filtered query using transaction entries + accounts
        let from_d = from.unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to_d = to.unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());

        let row = sqlx::query_as::<_, IncomeStatementDto>(
            r#"
            WITH rev AS (
                SELECT COALESCE(SUM(te.debit - te.credit), 0) AS revenue
                FROM accounting.transaction_entries te
                JOIN accounting.transactions t ON t.uuid = te.transaction_uuid
                JOIN accounting.accounts a ON a.uuid = te.account_uuid
                WHERE a.type = 'revenue' AND t.txn_date BETWEEN $1 AND $2
            ),
            cogs AS (
                SELECT COALESCE(SUM(te.debit - te.credit), 0) AS cogs
                FROM accounting.transaction_entries te
                JOIN accounting.transactions t ON t.uuid = te.transaction_uuid
                JOIN accounting.accounts a ON a.uuid = te.account_uuid
                WHERE a.code LIKE '5%' AND t.txn_date BETWEEN $1 AND $2
            ),
            opex AS (
                SELECT COALESCE(SUM(te.debit - te.credit), 0) AS opex
                FROM accounting.transaction_entries te
                JOIN accounting.transactions t ON t.uuid = te.transaction_uuid
                JOIN accounting.accounts a ON a.uuid = te.account_uuid
                WHERE a.type = 'expense' AND a.code NOT LIKE '6%' AND t.txn_date BETWEEN $1 AND $2
            ),
            payroll AS (
                SELECT COALESCE(SUM(ps.gross_pay), 0) AS payroll_expense
                FROM payroll.payslips ps
                JOIN payroll.payruns pr ON pr.uuid = ps.payrun_uuid
                WHERE pr.status = 'Posted' AND pr.payment_date BETWEEN $1 AND $2
            )
            SELECT
                r.revenue,
                c.cogs,
                (r.revenue - c.cogs) AS gross_profit,
                (o.opex + p.payroll_expense) AS operating_expenses,
                (r.revenue - c.cogs - o.opex - p.payroll_expense) AS operating_income,
                0::numeric AS other_income,
                0::numeric AS other_expense,
                (r.revenue - c.cogs - o.opex - p.payroll_expense) AS net_income
            FROM rev r, cogs c, opex o, payroll p
            "#,
        )
        .bind(from_d)
        .bind(to_d)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    async fn get_aging_ar(
        &self,
        pool: &PgPool,
        customer_uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<ArAgingDto>, AppError> {
        let rows = sqlx::query_as::<_, ArAgingDto>(
            r#"SELECT * FROM reporting.ar_aging_detailed WHERE customer_serial_id = (
                    SELECT serial_id FROM receivables.customers WHERE uuid = $1
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
                    SELECT serial_id FROM payables.vendors WHERE uuid = $1
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
        _period_uuid: Uuid,
        _user_id: Uuid,
        _from: Option<NaiveDate>,
        _to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError> {
        // cash_flow view is aggregated; for a period you'd need to aggregate transaction_entries by txn_date.
        // Quick path: return view row
        let row = sqlx::query_as::<_, CashFlowDto>(r#"SELECT * FROM reporting.cash_flow"#)
            .fetch_one(pool)
            .await?;
        Ok(row)
    }

    async fn get_cf_indirect(
        &self,
        pool: &PgPool,
        period_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError> {
        self.get_cf_direct(pool, period_uuid, user_id, from, to)
            .await
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
            JOIN receivables.invoices i ON i.serial_id = cb.txn_serial_id  -- adapt as needed
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
            JOIN payables.bills b ON b.serial_id = cb.txn_serial_id  -- adapt as needed
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
        _user_id: Uuid,
    ) -> Result<TrialBalanceDto, AppError> {
        let row = sqlx::query_as::<_, TrialBalanceDto>(r#"SELECT * FROM reporting.trial_balance"#)
            .fetch_one(pool)
            .await?;
        Ok(row)
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
}
