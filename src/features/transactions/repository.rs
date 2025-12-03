// src/features/transactions/repository.rs

use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::infrastructure::errors::AppError;
use crate::models::account::{
    CreateJournalEntry, JournalEntry, JournalEntryLine, JournalEntryWithLines,
    TransactionLineInput, UpdateJournalEntry,
};

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        input: &CreateJournalEntry,
    ) -> Result<JournalEntry, AppError>;

    async fn post_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<JournalEntry, AppError>;

    async fn update_unposted_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        input: &UpdateJournalEntry,
    ) -> Result<JournalEntry, AppError>;

    async fn delete_unposted_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_journal_entry_with_lines(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Option<JournalEntryWithLines>, AppError>;

    async fn list_journal_entries(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, AppError>;

    async fn void_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        reason: &str,
    ) -> Result<(), AppError>;

    async fn get_account_balance(
        &self,
        pool: &PgPool,
        account_uuid: Uuid,
    ) -> Result<BigDecimal, AppError>;
}
pub struct PostgresTransactionRepo;

impl PostgresTransactionRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransactionRepository for PostgresTransactionRepo {
    async fn create_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        input: &CreateJournalEntry,
    ) -> Result<JournalEntry, AppError> {
        let lines_json: Vec<serde_json::Value> = input
            .lines
            .iter()
            .map(|line| {
                json!({
                    "account_ref": line.account_uuid.to_string(),
                    "debit": line.debit,
                    "credit": line.credit,
                    "memo": line.memo
                })
            })
            .collect();

        let serial_id: (i64,) = sqlx::query_as(
            r#"
            SELECT accounting.post_transaction(
                $1, $2, $3, $4, $5, $6::jsonb
            )
            "#,
        )
        .bind(input.txn_date)
        .bind(&input.reference)
        .bind(&input.description)
        .bind(user_id)
        .bind(input.module.as_deref().unwrap_or("journal"))
        .bind(serde_json::to_value(lines_json).unwrap())
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let header: JournalEntry = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE serial_id = $1",
        )
        .bind(serial_id.0)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        // Override posted status if input specifies draft
        let final_header: JournalEntry = if !input.posted {
            sqlx::query_as::<_, JournalEntry>(
                "UPDATE accounting.transactions SET posted = false WHERE uuid = $1 RETURNING *",
            )
            .bind(header.uuid)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e))?
        } else {
            header
        };

        Ok(final_header)
    }

    async fn post_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<JournalEntry, AppError> {
        let entry = sqlx::query_as::<_, JournalEntry>(
            r#"
            UPDATE accounting.transactions
            SET posted = true
            WHERE uuid = $1 AND created_by = $2 AND posted = false
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        entry.ok_or(AppError::NotFound(
            "Draft entry not found or already posted".into(),
        ))
    }

    async fn update_unposted_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        input: &UpdateJournalEntry,
    ) -> Result<JournalEntry, AppError> {
        // Start a transaction
        let mut tx = pool.begin().await.map_err(|e| AppError::Database(e))?;

        // Verify entry exists and is unposted
        let _entry = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE uuid = $1 AND created_by = $2 AND posted = false"
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or(AppError::NotFound("Draft entry not found or already posted".into()))?;

        // If lines are provided, delete old lines and insert new ones
        if let Some(lines) = &input.lines {
            // Delete existing lines
            sqlx::query("DELETE FROM accounting.transaction_entries WHERE transaction_uuid = $1")
                .bind(uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e))?;

            // Insert new lines using stored procedure logic
            let _lines_json: Vec<serde_json::Value> = lines
                .iter()
                .map(|line| {
                    json!({
                        "account_ref": line.account_uuid.to_string(),
                        "debit": line.debit,
                        "credit": line.credit,
                        "memo": line.memo
                    })
                })
                .collect();

            // Validate balance
            let total_debit: BigDecimal = lines.iter().map(|l| l.debit.clone()).sum();
            let total_credit: BigDecimal = lines.iter().map(|l| l.credit.clone()).sum();
            if total_debit != total_credit {
                return Err(AppError::BadRequest(format!(
                    "Unbalanced entry: debit {total_debit} ≠ credit {total_credit}"
                )));
            }

            // Insert new lines (mimic post_transaction logic)
            let mut line_no = 1;
            for line in lines {
                sqlx::query(
                    r#"
                    INSERT INTO accounting.transaction_entries (
                        transaction_uuid, account_uuid, line_no, amount, debit, credit, memo
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(uuid)
                .bind(line.account_uuid)
                .bind(line_no)
                .bind(&line.debit - &line.credit)
                .bind(&line.debit)
                .bind(&line.credit)
                .bind(&line.memo)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e))?;
                line_no += 1;
            }
        }

        // Update header fields if provided
        let updated_entry = sqlx::query_as::<_, JournalEntry>(
            r#"
            UPDATE accounting.transactions
            SET
                txn_date = COALESCE($1, txn_date),
                reference = COALESCE($2, reference),
                description = COALESCE($3, description),
                module = COALESCE($4, module)
            WHERE uuid = $5 AND created_by = $6 AND posted = false
            RETURNING *
            "#,
        )
        .bind(&input.txn_date)
        .bind(&input.reference)
        .bind(&input.description)
        .bind(&input.module)
        .bind(uuid)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?;

        tx.commit().await.map_err(|e| AppError::Database(e))?;
        Ok(updated_entry)
    }

    async fn delete_unposted_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            "DELETE FROM accounting.transactions WHERE uuid = $1 AND created_by = $2 AND posted = false"
        )
        .bind(uuid)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Draft entry not found or already posted".into(),
            ));
        }

        Ok(())
    }

    async fn get_journal_entry_with_lines(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Option<JournalEntryWithLines>, AppError> {
        let header = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE uuid = $1 AND created_by = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let Some(header) = header else {
            return Ok(None);
        };

        let lines = sqlx::query_as::<_, JournalEntryLine>(
            "SELECT * FROM accounting.transaction_entries WHERE transaction_uuid = $1 ORDER BY line_no"
        )
        .bind(uuid)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(Some(JournalEntryWithLines { header, lines }))
    }

    async fn list_journal_entries(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, AppError> {
        let entries = sqlx::query_as::<_, JournalEntry>(
            r#"
            SELECT * FROM accounting.transactions
            WHERE created_by = $1
            ORDER BY txn_date DESC, created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(entries)
    }

    async fn void_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        reason: &str,
    ) -> Result<(), AppError> {
        let original = self
            .get_journal_entry_with_lines(pool, uuid, user_id)
            .await?
            .ok_or(AppError::NotFound("Entry not found".into()))?;

        let reversing_lines = original
            .lines
            .iter()
            .map(|line| TransactionLineInput {
                account_uuid: line.account_uuid,
                debit: line.credit.clone(),
                credit: line.debit.clone(),
                memo: Some(format!(
                    "REV: {} - {}",
                    original.header.description.as_deref().unwrap_or(""),
                    reason
                )),
            })
            .collect();

        let reversing: CreateJournalEntry = CreateJournalEntry {
            txn_date: Utc::now().date_naive(),
            reference: original.header.reference.clone(),
            description: Some(format!(
                "Reversal of #{} - {}",
                original.header.serial_id, reason
            )),
            module: Some("reversal".into()),
            posted: true,
            lines: reversing_lines,
        };

        self.create_journal_entry(pool, user_id, &reversing).await?;
        Ok(())
    }

    async fn get_account_balance(
        &self,
        pool: &PgPool,
        account_uuid: Uuid,
    ) -> Result<BigDecimal, AppError> {
        let bal: (Option<BigDecimal>,) = sqlx::query_as(
            r#"
            SELECT COALESCE(SUM(debit - credit), 0)
            FROM accounting.transaction_entries
            WHERE account_uuid = $1
            "#,
        )
        .bind(account_uuid)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(bal.0.unwrap_or(BigDecimal::zero()))
    }
}
