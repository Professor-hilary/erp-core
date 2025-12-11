// src/features/reports/services.rs
use crate::{
    features::reports::repository::ReportRepository,
    infrastructure::errors::AppError,
    models::reports::{
        ApAgingDto, ArAgingDto, BalanceSheetDto, CashFlowDto, CashbookRowDto, CustomerStatementDto,
        IncomeStatementDto, InventoryValuationDto, PayrollSummaryDto, TrialBalanceDto,
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
        user_id: Uuid,
    ) -> Result<BalanceSheetDto, AppError> {
        self.repo.get_balancesheet(tenant_pool, user_id).await
    }
    pub async fn income_statement(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<IncomeStatementDto, AppError> {
        self.repo.get_income(tenant_pool, user_id, from, to).await
    }
    pub async fn cf_direct(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError> {
        self.repo
            .get_cf_direct(tenant_pool, uuid, user_id, from, to)
            .await
    }
    pub async fn cf_indirect(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<CashFlowDto, AppError> {
        self.repo
            .get_cf_indirect(tenant_pool, uuid, user_id, from, to)
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
        user_id: Uuid,
    ) -> Result<TrialBalanceDto, AppError> {
        self.repo.get_trial_balance(tenant_pool, user_id).await
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
}
