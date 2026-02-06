// src/features/transactions/service.rs

use axum::Extension;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    features::transactions::repository::TransactionRepository,
    infrastructure::errors::AppError,
    middleware::auth::AuthenticatedTenant,
    models::transaction::{
        CreateJournalEntry, JournalEntry, JournalEntryWithLines, LedgerFilter, LedgerRowDto,
        TransactionLineInput, UpdateJournalEntry,
    },
};

pub struct TransactionService<R: TransactionRepository> {
    repo: R,
}

impl<R: TransactionRepository> TransactionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_journal_entry(
        &self,
        pool: &PgPool,
        Extension(tenant): Extension<AuthenticatedTenant>,
        journal: CreateJournalEntry,
    ) -> Result<JournalEntryWithLines, AppError> {
        // Validate balance
        let total_debit: BigDecimal = journal
            .lines
            .iter()
            .map(|l: &TransactionLineInput| l.debit.clone())
            .sum();
        let total_credit: BigDecimal = journal
            .lines
            .iter()
            .map(|l: &TransactionLineInput| l.credit.clone())
            .sum();
        let journal_date: NaiveDate = journal.txn_date;

        // Make sure total credit and total debit amounts balance
        if total_debit != total_credit {
            return Err(AppError::BadRequest(format!(
                "Unbalanced entry: debit {total_debit} ≠ credit {total_credit}"
            )));
        }

        // Make sure transaction is within financial period
        if journal_date < tenant.current_period_start || journal_date > tenant.current_period_end {
            return Err(AppError::BadRequest(format!(
                "Transaction date {} is outside current financial period {} to {}",
                journal_date, tenant.current_period_start, tenant.current_period_end
            )));
        }

        // Make sure the financial period is not locked
        if tenant.period_is_locked {
            return Err(AppError::Forbidden(format!(
                "Cannot post to a locked financial period"
            )));
        }

        // Make sure at least one debit and one credit transaction line exist
        if journal.lines.len() < 2 {
            return Err(AppError::BadRequest("Entry must have ≥2 lines".into()));
        }

        let header: JournalEntry = self
            .repo
            .create_journal_entry(pool, tenant.user_id, &journal)
            .await?;
        self.repo
            .get_journal_entry_with_lines(pool, header.uuid, tenant.user_id)
            .await?
            .ok_or(AppError::NotFound(
                "Failed to retrieve created entry".into(),
            ))
    }

    pub async fn post_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        uuid: Uuid,
    ) -> Result<JournalEntryWithLines, AppError> {
        let header = self.repo.post_journal_entry(pool, uuid, user_id).await?;
        self.repo
            .get_journal_entry_with_lines(pool, header.uuid, user_id)
            .await?
            .ok_or(AppError::NotFound("Failed to retrieve posted entry".into()))
    }

    pub async fn update_unposted_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        uuid: Uuid,
        journal: UpdateJournalEntry,
    ) -> Result<JournalEntryWithLines, AppError> {
        // Validate new lines if provided
        if let Some(lines) = &journal.lines {
            let total_debit: BigDecimal = lines.iter().map(|l| l.debit.clone()).sum();
            let total_credit: BigDecimal = lines.iter().map(|l| l.credit.clone()).sum();
            if total_debit != total_credit {
                return Err(AppError::BadRequest(format!(
                    "Unbalanced entry: debit {total_debit} ≠ credit {total_credit}"
                )));
            }
            if lines.len() < 2 {
                return Err(AppError::BadRequest("Entry must have ≥2 lines".into()));
            }
        }

        let header = self
            .repo
            .update_unposted_journal_entry(pool, uuid, user_id, &journal)
            .await?;
        self.repo
            .get_journal_entry_with_lines(pool, header.uuid, user_id)
            .await?
            .ok_or(AppError::NotFound(
                "Failed to retrieve updated entry".into(),
            ))
    }

    pub async fn delete_unposted_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        uuid: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_unposted_journal_entry(pool, uuid, user_id)
            .await
    }

    pub async fn void_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        uuid: Uuid,
        reason: String,
    ) -> Result<(), AppError> {
        self.repo
            .void_journal_entry(pool, uuid, user_id, &reason)
            .await
    }

    #[allow(unused)]
    pub async fn get_account_balance(
        &self,
        pool: &PgPool,
        account_uuid: Uuid,
    ) -> Result<BigDecimal, AppError> {
        self.repo.get_account_balance(pool, account_uuid).await
    }

    pub async fn list_full_journal_entries(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JournalEntryWithLines>, AppError> {
        self.repo.get_all_journal_with_lines(pool, user_id).await
    }

    pub async fn list_journal_entries(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, AppError> {
        self.repo
            .list_journal_entries(pool, user_id, limit, offset)
            .await
    }

    pub async fn get_journal_entry_with_lines(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Option<JournalEntryWithLines>, AppError> {
        self.repo
            .get_journal_entry_with_lines(pool, uuid, user_id)
            .await
    }

    pub async fn fetch_ledger(
        &self,
        pool: &PgPool,
        filter: LedgerFilter,
    ) -> Result<Vec<LedgerRowDto>, AppError> {
        self.repo.fetch_account_ledger(pool, filter).await
    }
}
