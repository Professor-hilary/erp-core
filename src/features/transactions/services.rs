// src/features/transactions/service.rs

use bigdecimal::BigDecimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::features::transactions::repository::TransactionRepository;
use crate::infrastructure::errors::AppError;
use crate::models::account::{
    CreateJournalEntry, JournalEntry, JournalEntryWithLines, LedgerFilter, LedgerRowDto,
    UpdateJournalEntry,
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
        user_id: Uuid,
        input: CreateJournalEntry,
    ) -> Result<JournalEntryWithLines, AppError> {
        // Validate balance
        let total_debit: BigDecimal = input.lines.iter().map(|l| l.debit.clone()).sum();
        let total_credit: BigDecimal = input.lines.iter().map(|l| l.credit.clone()).sum();
        if total_debit != total_credit {
            return Err(AppError::BadRequest(format!(
                "Unbalanced entry: debit {total_debit} ≠ credit {total_credit}"
            )));
        }

        if input.lines.len() < 2 {
            return Err(AppError::BadRequest("Entry must have ≥2 lines".into()));
        }

        let header: JournalEntry = self
            .repo
            .create_journal_entry(pool, user_id, &input)
            .await?;
        self.repo
            .get_journal_entry_with_lines(pool, header.uuid, user_id)
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
        input: UpdateJournalEntry,
    ) -> Result<JournalEntryWithLines, AppError> {
        // Validate new lines if provided
        if let Some(lines) = &input.lines {
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
            .update_unposted_journal_entry(pool, uuid, user_id, &input)
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
