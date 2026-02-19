use std::sync::Arc;

// src/features/accounts/repositories.rs
use crate::{
    interface::api::errors::AppError,
    models::{
        account::{Account, CreateAccount},
        dto::FinancialPeriodDto,
    },
    state::AppState,
};
use async_trait::async_trait;
use chrono::NaiveDate;
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
    // #[allow(unused)]
    // async fn update_balance(
    //     &self,
    //     pool: &PgPool,
    //     id: sqlx::types::Uuid,
    //     delta: BigDecimal,
    // ) -> Result<(), AppError>;
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
        uuid: Uuid,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<(), AppError>;
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
                name, category, code, parent_code, normal_balance, is_contra
            ) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(&acc.name)
        .bind(acc.category.to_lowercase())
        .bind(&acc.code)
        .bind(&acc.parent_code)
        .bind(&acc.normal_balance)
        .bind(acc.is_contra)
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

    // async fn update_balance(
    //     &self,
    //     pool: &PgPool,
    //     id: sqlx::types::Uuid,
    //     delta: BigDecimal,
    // ) -> Result<(), AppError> {
    //     sqlx::query("UPDATE accounting.accounts SET balance = balance + $1 WHERE serial_id = $2")
    //         .bind(delta)
    //         .bind(id)
    //         .execute(pool)
    //         .await?;
    //     Ok(())
    // }

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
        user_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM accounting.accounts WHERE id = $1 AND user_id = $2")
            .bind(uuid)
            .bind(user_id)
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
        let periods: Vec<FinancialPeriodDto> = sqlx::query_as::<_, FinancialPeriodDto>(
            r#"
                SELECT
                    uuid,
                    start_date,
                    end_date,
                    name,
                    is_open,
                    is_locked
                FROM accounting.financial_periods
                ORDER BY start_date DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(periods)
    }

    async fn close_period(
        &self,
        state: Arc<AppState>,
        uuid: Uuid,
        pool: &PgPool,
        company_id: Uuid,
    ) -> Result<(), AppError> {
        let updated = sqlx::query(
            r#"
                UPDATE accounting.financial_periods SET is_open = false,
                    is_locked = true, updated_at = NOW()
                    WHERE uuid = $1 AND is_open = true
            "#,
        )
        .bind(uuid)
        .execute(pool)
        .await?
        .rows_affected();

        if updated == 0 {
            return Err(AppError::NotFound(
                "Period not found or closed already".into(),
            ));
        }

        // Invalidate cached period
        state.period_cache.invalidate(&company_id).await;

        Ok(())
    }
}
