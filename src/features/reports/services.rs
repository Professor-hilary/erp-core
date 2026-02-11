// src/features/reports/services.rs
use crate::{
    features::reports::repository::{ForceReload, ReportRepository},
    interface::api::errors::AppError,
    models::reports::{
        ApAgingDto, ArAgingDto, BalanceSheetCompareRow, BalanceSheetRow, CashFlowRow,
        CashbookRowDto, CashflowGroup, CustomerStatementDto, EquityChangeRow, IncomeStatementRow,
        InventoryValuationDto, PayrollSummaryDto, TrialBalanceRow,
    },
};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ReportService<R: ReportRepository> {
    repo: R,
}

impl<R: ReportRepository> ReportService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn balancesheet(
        &self,
        tenant_pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<BalanceSheetRow>, AppError> {
        self.repo.get_balancesheet(tenant_pool, as_of).await
    }
    pub async fn balancesheet_compare(
        &self,
        tenant_pool: &PgPool,
        as_of_1: NaiveDate,
        as_of_2: NaiveDate,
    ) -> Result<Vec<BalanceSheetCompareRow>, AppError> {
        self.repo
            .get_balancesheet_comparison(tenant_pool, as_of_1, as_of_2)
            .await
    }
    pub async fn income_statement(
        &self,
        tenant_pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<IncomeStatementRow>, AppError> {
        self.repo.get_income(tenant_pool, start, end).await
    }
    pub async fn cf_direct(
        &self,
        tenant_pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CashflowGroup>, AppError> {
        self.repo.get_cf_direct(tenant_pool, start, end).await
    }
    pub async fn cf_indirect(
        &self,
        tenant_pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<CashFlowRow>, AppError> {
        self.repo.get_cf_indirect(tenant_pool, start, end).await
    }
    pub async fn change_of_equity(
        &self,
        tenant_pool: &PgPool,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<EquityChangeRow>, AppError> {
        self.repo
            .get_change_of_equity(tenant_pool, start, end)
            .await
    }
    pub async fn aging_ar(
        &self,
        tenant_pool: &PgPool,
        customer_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ArAgingDto>, AppError> {
        self.repo
            .get_aging_ar(tenant_pool, customer_uuid, user_id)
            .await
    }

    pub async fn aging_ap(
        &self,
        tenant_pool: &PgPool,
        vendor_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ApAgingDto>, AppError> {
        self.repo
            .get_aging_ap(tenant_pool, vendor_uuid, user_id)
            .await
    }

    pub async fn purchase_report(
        &self,
        tenant_pool: &PgPool,
        vendor_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError> {
        self.repo
            .get_purchase_report(tenant_pool, vendor_uuid, user_id, from, to)
            .await
    }

    pub async fn sales_report(
        &self,
        tenant_pool: &PgPool,
        customer_uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CashbookRowDto>, AppError> {
        self.repo
            .get_sales_report(tenant_pool, customer_uuid, user_id, from, to)
            .await
    }

    pub async fn trial_balance(
        &self,
        tenant_pool: &PgPool,
        as_of: NaiveDate,
    ) -> Result<Vec<TrialBalanceRow>, AppError> {
        self.repo.get_trial_balance(tenant_pool, as_of).await
    }

    pub async fn inventory_valuation(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<InventoryValuationDto, AppError> {
        self.repo
            .get_inventory_valuation(tenant_pool, user_id)
            .await
    }

    pub async fn customer_statement(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<CustomerStatementDto, AppError> {
        self.repo.get_customer_report(tenant_pool, user_id).await
    }

    pub async fn payroll_statement(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<PayrollSummaryDto, AppError> {
        self.repo.get_payroll(tenant_pool, user_id).await
    }

    pub async fn refresh_all(
        &self,
        tenant_pool: &PgPool,
        payload: &ForceReload,
    ) -> Result<(), AppError> {
        self.repo.refresh_reports(tenant_pool, payload).await
    }
}
