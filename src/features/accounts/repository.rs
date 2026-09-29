// src/features/accounts/repositories.rs
use std::sync::Arc;

use crate::{
    interface::api::errors::AppError,
    models::{account::*, dto::FinancialPeriodDto},
    state::AppState,
};
// use async_trait::async_trait;
use async_trait::async_trait;
use bigdecimal::{BigDecimal, Zero};
use chrono::{Months, NaiveDate};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// Trait for abstraction (injectable for testing)
#[async_trait]
pub trait AccountRepository: Send + Sync {
    // fn pool(&self) -> &PgPool;
    async fn create(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        acc: &CreateAccount,
    ) -> Result<Account, AppError>;
    async fn find_by_serial_id(
        &self,
        pool: &PgPool,
        id: i64,
        user_id: Uuid,
    ) -> Result<Option<Account>, AppError>;
    async fn find_by_uuid(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Account>, AppError>;
    // find_by_user() is useful later
    async fn find_by_user(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Account>, AppError>;

    async fn update_account_info(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        updates: &CreateAccount,
    ) -> Result<Account, AppError>;
    async fn delete_account(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;
    async fn create_initial_period(
        &self,
        pool: &PgPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError>;
    async fn list_periods(&self, pool: &PgPool) -> Result<Vec<FinancialPeriodDto>, AppError>;
    async fn close_period(
        &self,
        state: Arc<AppState>,
        period_uuid: Uuid,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn create_currency(
        &self,
        pool: &PgPool,
        payload: &CreateCurrency,
    ) -> Result<Currency, AppError>;

    async fn list_currencies(
        &self,
        pool: &PgPool,
        active_only: bool,
    ) -> Result<Vec<Currency>, AppError>;

    async fn get_currency_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Currency, AppError>;

    async fn get_currency_by_code(&self, pool: &PgPool, code: &str) -> Result<Currency, AppError>;

    async fn update_currency(
        &self,
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateCurrency,
    ) -> Result<Currency, AppError>;

    async fn upsert_exchange_rate(
        &self,
        pool: &PgPool,
        payload: &UpsertExchangeRate,
    ) -> Result<ExchangeRate, AppError>;

    async fn list_exchange_rates(
        &self,
        pool: &PgPool,
        from_currency_id: Option<Uuid>,
        to_currency_id: Option<Uuid>,
    ) -> Result<Vec<ExchangeRate>, AppError>;

    /// Rate on or before `on` (nearest previous).
    async fn get_rate(
        &self,
        pool: &PgPool,
        from_currency_id: Uuid,
        to_currency_id: Uuid,
        on: NaiveDate,
    ) -> Result<BigDecimal, AppError>;
}

// Concrete impls
pub struct PostgresAccountRepo;

impl PostgresAccountRepo {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepo {
    /// # Purpose
    /// Create a new user and insert into the database. User may be employee, admin, or owner.
    /// Purpose of user is authentication and authorization to access and use the application.
    /// User will be used to perform sign in/signup, DBMS CRUD functions and any relevant jobs.
    ///
    /// # Errors
    ///
    /// This function will return an error if user cannot be created.
    async fn create(
        &self,
        pool: &PgPool,
        _user_id: Uuid,
        acc: &CreateAccount,
    ) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "INSERT INTO accounting.accounts (
                name, category, code, parent_code, normal_balance, is_contra, cash_flow_category
            ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        )
        .bind(&acc.name)
        .bind(acc.category.to_lowercase())
        .bind(&acc.code)
        .bind(&acc.parent_code)
        .bind(&acc.normal_balance)
        .bind(acc.is_contra)
        .bind(&acc.cash_flow_category)
        .fetch_one(pool)
        .await?;
        Ok(account)
    }

    async fn find_by_serial_id(
        &self,
        pool: &PgPool,
        id: i64,
        user_id: Uuid,
    ) -> Result<Option<Account>, AppError> {
        let account =
            sqlx::query_as::<_, Account>("SELECT * FROM accounting.accounts WHERE serial_id = $1")
                .bind(id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;
        Ok(account)
    }

    async fn find_by_uuid(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Account>, AppError> {
        let account =
            sqlx::query_as::<_, Account>("SELECT * FROM accounting.accounts WHERE uuid = $1")
                .bind(uuid)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;
        Ok(account)
    }

    async fn find_by_user(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<Account>, AppError> {
        let accounts: Vec<Account> =
            sqlx::query_as::<_, Account>("SELECT * FROM accounting.accounts")
                .bind(user_id)
                .fetch_all(pool)
                .await?;
        Ok(accounts)
    }

    async fn update_account_info(
        &self,
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        updates: &CreateAccount,
    ) -> Result<Account, AppError> {
        let account = sqlx::query_as::<_, Account>(
            "UPDATE accounting.accounts SET name = $1, category = $2 WHERE id = $3 AND user_id = $4 RETURNING *",
        )
        .bind(&updates.name)
        .bind(&updates.category)
        .bind(id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.as_database_error()
                .and_then(|db| db.constraint())
                .map(|c| c == "accounts_type_check")
                .unwrap_or(false)
            {
                AppError::BadRequest("Invalid account type".into())
            } else {
                AppError::Database(e)
            }
        })?;

        Ok(account)
    }

    async fn delete_account(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM accounting.accounts WHERE uuid = $1")
            .bind(uuid)
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn create_initial_period(
        &self,
        pool: &PgPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO accounting.financial_periods (
                start_date, end_date, is_open, is_locked
            ) VALUES ($1, $2, true, false)
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    async fn list_periods(&self, pool: &PgPool) -> Result<Vec<FinancialPeriodDto>, AppError> {
        let periods = sqlx::query_as::<_, FinancialPeriodDto>(
            r#"
                SELECT
                    uuid, start_date, end_date, name, is_open, is_locked
                FROM accounting.financial_periods
                ORDER BY start_date DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(periods)
    }

    // In src/features/accounts/repository.rs
    // (inside the `impl AccountRepository for PostgresAccountRepo` block)

    async fn close_period(
        &self,
        state: Arc<AppState>,
        period_uuid: Uuid,
        pool: &PgPool,
        company_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        // ──────────────────────────────────────────────────────────────
        // 1. Load & validate the period
        // ──────────────────────────────────────────────────────────────
        let (start_date, end_date, is_open, is_locked): (
            chrono::NaiveDate,
            chrono::NaiveDate,
            bool,
            bool,
        ) = sqlx::query_as(
            r#"
        SELECT start_date, end_date, is_open, COALESCE(is_locked, false)
        FROM accounting.financial_periods
        WHERE uuid = $1
        "#,
        )
        .bind(period_uuid)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Financial period not found".into()))?;

        if !is_open || is_locked {
            return Err(AppError::BadRequest(
                "Period is already closed or locked".into(),
            ));
        }

        // ──────────────────────────────────────────────────────────────
        // 2. Snapshot current balances (BEFORE we zero anything)
        //    - For income/expense  → this period's activity
        //    - For asset/liability/equity → ending balance (BS)
        // ──────────────────────────────────────────────────────────────
        sqlx::query(
            r#"
        INSERT INTO accounting.period_account_balances (
            period_id, account_uuid, debit_total, credit_total, net_balance, category
        )
        SELECT
            $1,
            a.uuid,
            CASE WHEN a.current_balance > 0 THEN a.current_balance ELSE 0 END,
            CASE WHEN a.current_balance < 0 THEN -a.current_balance ELSE 0 END,
            a.current_balance,
            a.category
        FROM accounting.accounts a
        WHERE a.is_active = true
          AND a.current_balance <> 0
        ON CONFLICT (period_id, account_uuid) DO UPDATE SET
            debit_total  = EXCLUDED.debit_total,
            credit_total = EXCLUDED.credit_total,
            net_balance  = EXCLUDED.net_balance,
            category     = EXCLUDED.category
        "#,
        )
        .bind(period_uuid)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        // ──────────────────────────────────────────────────────────────
        // 3. Load nominal accounts that still have a balance
        // ──────────────────────────────────────────────────────────────
        #[derive(sqlx::FromRow)]
        struct Nominal {
            uuid: Uuid,
            code: String,
            name: String,
            category: String,
            current_balance: BigDecimal, // debit − credit (same sign as trigger)
        }

        let nominals: Vec<Nominal> = sqlx::query_as(
            r#"
        SELECT uuid, code, name, category, current_balance
        FROM accounting.accounts
        WHERE category IN ('income', 'expense')
          AND is_active = true
          AND current_balance <> 0
        ORDER BY code
        "#,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        // ──────────────────────────────────────────────────────────────
        // 4. Resolve equity target (Owner's Equity)
        // ──────────────────────────────────────────────────────────────
        let (equity_uuid,): (Uuid,) = sqlx::query_as(
            r#"
        SELECT uuid FROM accounting.accounts
        WHERE code = '310000'
           OR (category = 'equity' AND name ILIKE '%retained%')
           OR (category = 'equity' AND name ILIKE '%owner%')
        ORDER BY CASE WHEN code = '310000' THEN 0 ELSE 1 END
        LIMIT 1
        "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::Internal("No Owner's Equity / Retained Earnings account found in COA".into())
        })?;

        // ──────────────────────────────────────────────────────────────
        // 5. Build closing journal lines
        //    current_balance is stored as (debit − credit)
        //    - positive → net debit  (typical expense)
        //    - negative → net credit (typical income)
        // ──────────────────────────────────────────────────────────────
        let mut lines: Vec<serde_json::Value> = Vec::new();
        let mut net_income = BigDecimal::zero(); // positive = profit

        for acc in &nominals {
            let bal = &acc.current_balance;
            if bal.is_zero() {
                continue;
            }

            let amount = bal.abs();

            if bal > &BigDecimal::zero() {
                // Net debit balance → credit the account to zero it
                // (expense or unusual income debit balance)
                lines.push(json!({
                    "account_ref": acc.uuid.to_string(),
                    "debit": BigDecimal::zero(),
                    "credit": amount,
                    "memo": format!("Close {} ({})", acc.name, acc.code)
                }));
                // Debit equity (reduces equity)
                lines.push(json!({
                    "account_ref": equity_uuid.to_string(),
                    "debit": amount,
                    "credit": BigDecimal::zero(),
                    "memo": format!("Close {}", acc.code)
                }));
                if acc.category == "expense" {
                    net_income -= amount;
                } else {
                    // income with debit balance is unusual; treat as reduction of profit
                    net_income -= amount;
                }
            } else {
                // Net credit balance → debit the account to zero it
                // (income)
                lines.push(json!({
                    "account_ref": acc.uuid.to_string(),
                    "debit": amount,
                    "credit": BigDecimal::zero(),
                    "memo": format!("Close {} ({})", acc.name, acc.code)
                }));
                // Credit equity (increases equity = profit)
                lines.push(json!({
                    "account_ref": equity_uuid.to_string(),
                    "debit": BigDecimal::zero(),
                    "credit": amount,
                    "memo": format!("Close {}", acc.code)
                }));
                net_income += amount;
            }
        }

        // ──────────────────────────────────────────────────────────────
        // 6. Post the closing journal (if anything to close)
        // ──────────────────────────────────────────────────────────────
        if !lines.is_empty() {
            let _serial: (i64,) = sqlx::query_as(
                r#"
            SELECT accounting.post_transaction(
                $1, $2, $3, $4, $5, $6::jsonb
            )
            "#,
            )
            .bind(format!("CLOSE-{}", end_date.format("%Y%m%d")))
            .bind(format!(
                "Period close {} → {} (net income {})",
                start_date, end_date, net_income
            ))
            .bind(user_id)
            .bind("closing")
            .bind(end_date) // dated on the last day of the period
            .bind(serde_json::to_value(&lines).map_err(|e| {
                AppError::Internal(format!("Failed to serialise closing lines: {e}"))
            })?)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
        }

        // ──────────────────────────────────────────────────────────────
        // 7. Lock the period
        // ──────────────────────────────────────────────────────────────
        let updated = sqlx::query(
            r#"
        UPDATE accounting.financial_periods
        SET is_open   = false,
            is_locked = true,
            updated_at = NOW()
        WHERE uuid = $1 AND is_open = true
        "#,
        )
        .bind(period_uuid)
        .execute(pool)
        .await
        .map_err(AppError::Database)?
        .rows_affected();

        if updated == 0 {
            return Err(AppError::NotFound(
                "Period not found or already closed".into(),
            ));
        }

        // ──────────────────────────────────────────────────────────────
        // 8. Open the next period (simple +1 month for now)
        //    Later: read company.period_type and calculate correctly
        // ──────────────────────────────────────────────────────────────
        let next_start = end_date
            .succ_opt()
            .ok_or_else(|| AppError::Internal("Invalid period end date".into()))?;

        let next_end = next_start
            .checked_add_months(Months::new(1))
            .and_then(|d| d.pred_opt())
            .unwrap_or(next_start);

        sqlx::query(
            r#"
        INSERT INTO accounting.financial_periods (
            start_date, end_date, is_open, is_locked, name
        )
        VALUES ($1, $2, true, false, $3)
        ON CONFLICT (start_date, end_date) DO NOTHING
        "#,
        )
        .bind(next_start)
        .bind(next_end)
        .bind(format!("{} to {}", next_start, next_end))
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        // ──────────────────────────────────────────────────────────────
        // 9. Invalidate cache
        // ──────────────────────────────────────────────────────────────
        state.period_cache.invalidate(&company_id).await;

        Ok(())
    }

    async fn create_currency(
        &self,
        pool: &PgPool,
        payload: &CreateCurrency,
    ) -> Result<Currency, AppError> {
        let row = sqlx::query_as::<_, Currency>(
            r#"
            INSERT INTO accounting.currencies (code, name, decimal_places, is_active)
            VALUES (UPPER($1), $2, COALESCE($3, 2), COALESCE($4, true))
            RETURNING *
            "#,
        )
        .bind(payload.code.trim())
        .bind(&payload.name)
        .bind(payload.decimal_places)
        .bind(payload.is_active)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    async fn list_currencies(
        &self,
        pool: &PgPool,
        active_only: bool,
    ) -> Result<Vec<Currency>, AppError> {
        let rows = if active_only {
            sqlx::query_as::<_, Currency>(
                r#"
                SELECT * FROM accounting.currencies
                WHERE is_active = true
                ORDER BY code
                "#,
            )
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, Currency>(
                r#"
                SELECT * FROM accounting.currencies
                ORDER BY code
                "#,
            )
            .fetch_all(pool)
            .await
        }
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    async fn get_currency_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Currency, AppError> {
        sqlx::query_as::<_, Currency>(r#"SELECT * FROM accounting.currencies WHERE id = $1"#)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound("Currency not found".into()))
    }

    async fn get_currency_by_code(&self, pool: &PgPool, code: &str) -> Result<Currency, AppError> {
        sqlx::query_as::<_, Currency>(
            r#"SELECT * FROM accounting.currencies WHERE code = UPPER($1)"#,
        )
        .bind(code.trim())
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("Currency code not found: {code}")))
    }

    async fn update_currency(
        &self,
        pool: &PgPool,
        id: Uuid,
        payload: &UpdateCurrency,
    ) -> Result<Currency, AppError> {
        let row = sqlx::query_as::<_, Currency>(
            r#"
            UPDATE accounting.currencies
            SET
                name           = COALESCE($2, name),
                decimal_places = COALESCE($3, decimal_places),
                is_active      = COALESCE($4, is_active)
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.name)
        .bind(payload.decimal_places)
        .bind(payload.is_active)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Currency not found".into()))?;

        Ok(row)
    }

    async fn upsert_exchange_rate(
        &self,
        pool: &PgPool,
        payload: &UpsertExchangeRate,
    ) -> Result<ExchangeRate, AppError> {
        if payload.from_currency_id == payload.to_currency_id {
            return Err(AppError::BadRequest(
                "from_currency and to_currency must differ".into(),
            ));
        }

        let row = sqlx::query_as::<_, ExchangeRate>(
            r#"
            INSERT INTO accounting.exchange_rates (
                from_currency_id, to_currency_id, rate_date, rate, source
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (from_currency_id, to_currency_id, rate_date)
            DO UPDATE SET
                rate   = EXCLUDED.rate,
                source = EXCLUDED.source
            RETURNING *
            "#,
        )
        .bind(payload.from_currency_id)
        .bind(payload.to_currency_id)
        .bind(payload.rate_date)
        .bind(&payload.rate)
        .bind(&payload.source)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    async fn list_exchange_rates(
        &self,
        pool: &PgPool,
        from_currency_id: Option<Uuid>,
        to_currency_id: Option<Uuid>,
    ) -> Result<Vec<ExchangeRate>, AppError> {
        let rows = sqlx::query_as::<_, ExchangeRate>(
            r#"
            SELECT *
            FROM accounting.exchange_rates
            WHERE ($1::uuid IS NULL OR from_currency_id = $1)
              AND ($2::uuid IS NULL OR to_currency_id = $2)
            ORDER BY rate_date DESC
            "#,
        )
        .bind(from_currency_id)
        .bind(to_currency_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    async fn get_rate(
        &self,
        pool: &PgPool,
        from_currency_id: Uuid,
        to_currency_id: Uuid,
        on: NaiveDate,
    ) -> Result<BigDecimal, AppError> {
        if from_currency_id == to_currency_id {
            return Ok(BigDecimal::from(1));
        }

        let rate: Option<(BigDecimal,)> = sqlx::query_as(
            r#"
            SELECT rate
            FROM accounting.exchange_rates
            WHERE from_currency_id = $1
              AND to_currency_id   = $2
              AND rate_date       <= $3
            ORDER BY rate_date DESC
            LIMIT 1
            "#,
        )
        .bind(from_currency_id)
        .bind(to_currency_id)
        .bind(on)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        rate.map(|r| r.0).ok_or_else(|| {
            AppError::NotFound(format!("No exchange rate found for pair on or before {on}"))
        })
    }
}
