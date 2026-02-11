// src/features/transactions/repository.rs
use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use chrono::Utc;
use serde_json::json;
use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::{
    interface::api::errors::AppError,
    models::transaction::{
        CreateJournalEntry, JournalEntry, JournalEntryLine, JournalEntryWithLines, LedgerFilter,
        LedgerRowDto, TransactionLineInput, UpdateJournalEntry,
    },
};

#[async_trait]
pub trait TransactionRepository: Send + Sync {
    async fn create_journal_entry(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        journal: &CreateJournalEntry,
    ) -> Result<JournalEntry, AppError>;

    async fn fetch_account_ledger(
        &self,
        pool: &PgPool,
        filter: LedgerFilter,
    ) -> Result<Vec<LedgerRowDto>, AppError>;

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
        journal: &UpdateJournalEntry,
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

    async fn get_all_journal_with_lines(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JournalEntryWithLines>, AppError>;

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
        journal: &CreateJournalEntry,
    ) -> Result<JournalEntry, AppError> {
        let lines_json: Vec<serde_json::Value> = journal
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

        tracing::debug!("Before posting to accounting.post_transaction(...)");

        // validate transaction date, should be in the financial period
        let serial_id: (i64,) = sqlx::query_as(
            r#"
            SELECT accounting.post_transaction(
                $1, $2, $3, $4, $5, $6::jsonb
            )
            "#,
        )
        .bind(&journal.reference)
        .bind(&journal.description)
        .bind(user_id)
        .bind(journal.module.as_deref().unwrap_or("journal"))
        .bind(journal.txn_date)
        .bind(serde_json::to_value(lines_json).unwrap())
        .fetch_one(pool)
        .await
        .map_err(|e: Error| AppError::Database(e))?;

        tracing::debug!("After posting to accounting.post_transaction(...)");

        let header: JournalEntry = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE serial_id = $1",
        )
        .bind(serial_id.0)
        .fetch_one(pool)
        .await
        .map_err(|e: Error| AppError::Database(e))?;

        tracing::debug!("Setting posted flag");

        // Override posted status if journal specifies draft
        let final_header: JournalEntry = if !journal.posted {
            sqlx::query_as::<_, JournalEntry>(
                "UPDATE accounting.transactions SET posted = false WHERE uuid = $1 RETURNING *",
            )
            .bind(header.uuid)
            .fetch_one(pool)
            .await
            .map_err(|e: Error| AppError::Database(e))?
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
        let entry: Option<JournalEntry> = sqlx::query_as::<_, JournalEntry>(
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
        .map_err(|e: Error| AppError::Database(e))?;

        entry.ok_or(AppError::NotFound(
            "Draft entry not found or already posted".into(),
        ))
    }

    async fn update_unposted_journal_entry(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        journal: &UpdateJournalEntry,
    ) -> Result<JournalEntry, AppError> {
        // Start a transaction
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool
            .begin()
            .await
            .map_err(|e: Error| AppError::Database(e))?;

        // Verify entry exists and is unposted
        let _entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE uuid = $1 AND created_by = $2 AND posted = false"
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or(AppError::NotFound("Draft entry not found or already posted".into()))?;

        // If lines are provided, delete old lines and insert new ones
        if let Some(lines) = &journal.lines {
            // Delete existing lines
            sqlx::query("DELETE FROM accounting.transaction_entries WHERE transaction_uuid = $1")
                .bind(uuid)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e))?;

            // Insert new lines using stored procedure logic
            let _lines_json: Vec<serde_json::Value> = lines
                .iter()
                .map(|line: &TransactionLineInput| {
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
            let mut line_no: i32 = 1;
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
        let updated_entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(
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
        .bind(&journal.txn_date)
        .bind(&journal.reference)
        .bind(&journal.description)
        .bind(&journal.module)
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
        let result: sqlx::postgres::PgQueryResult = sqlx::query(
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
        let header: Option<JournalEntry> = sqlx::query_as::<_, JournalEntry>(
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

        let lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(
            r#"
            SELECT
                te.uuid,
                te.transaction_uuid,
                te.account_uuid,
                te.line_no,
                te.amount,
                te.debit,
                te.credit,
                te.memo,
                a.name      AS account_name,
                a.code      AS account_code,
                a.category  AS account_category
            FROM accounting.transaction_entries te
            LEFT JOIN accounting.accounts a ON te.account_uuid = a.uuid
            WHERE te.transaction_uuid = $1
            ORDER BY te.line_no
            "#,
        )
        .bind(uuid)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(Some(JournalEntryWithLines { header, lines }))
    }

    async fn get_all_journal_with_lines(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JournalEntryWithLines>, AppError> {
        let headers: Vec<JournalEntry> = sqlx::query_as::<_, JournalEntry>(
            "SELECT * FROM accounting.transactions WHERE created_by = $1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        let mut transactions: Vec<JournalEntryWithLines> = Vec::with_capacity(headers.len());

        for header in headers {
            let lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(
                r#"
                SELECT
                    te.uuid,
                    te.transaction_uuid,
                    te.account_uuid,
                    te.line_no,
                    te.amount,
                    te.debit,
                    te.credit,
                    te.memo,
                    a.name      AS account_name,
                    a.code      AS account_code,
                    a.category  AS account_category
                FROM accounting.transaction_entries te
                LEFT JOIN accounting.accounts a ON te.account_uuid = a.uuid
                WHERE te.transaction_uuid = $1
                ORDER BY te.line_no
                "#,
            )
            .bind(header.uuid)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Database(e))?;

            transactions.push(JournalEntryWithLines { header, lines });
        }
        Ok(transactions)
    }

    async fn list_journal_entries(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, AppError> {
        let entries: Vec<JournalEntry> = sqlx::query_as::<_, JournalEntry>(
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
        let original: JournalEntryWithLines = self
            .get_journal_entry_with_lines(pool, uuid, user_id)
            .await?
            .ok_or(AppError::NotFound("Entry not found".into()))?;

        let reversing_lines: Vec<TransactionLineInput> = original
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

    async fn fetch_account_ledger(
        &self,
        pool: &PgPool,
        filter: LedgerFilter,
    ) -> Result<Vec<LedgerRowDto>, AppError> {
        let rows = sqlx::query_as::<_, LedgerRowDto>(
            r#"
            SELECT
                t.uuid          AS transaction_uuid,
                t.serial_id     AS transaction_serial_id,
                te.uuid         AS entry_uuid,
                te.serial_id    AS entry_serial_id,

                t.txn_date,
                t.reference,
                t.description,

                te.debit,
                te.credit,
                te.amount,
                te.memo,
                te.debit,
                te.created_at,

                SUM(te.amount) OVER (
                    ORDER BY t.txn_date, te.serial_id
                    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
                ) AS running_balance

            FROM accounting.transaction_entries te
            JOIN accounting.transactions t
                ON t.uuid = te.transaction_uuid

            WHERE te.account_uuid = $1
                AND ($2::date IS NULL OR t.txn_date >= $2)
                AND ($3::date IS NULL OR t.txn_date <= $3)
                AND ($4::boolean = FALSE OR t.posted = TRUE)

            ORDER BY t.txn_date, te.serial_id
        "#,
        )
        .bind(filter.account_uuid)
        .bind(filter.from_date)
        .bind(filter.to_date)
        .bind(filter.posted_only)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
