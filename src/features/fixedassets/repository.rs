use crate::{interface::api::errors::AppError, models::fixed_assets::*};

use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use sqlx::{PgPool, postgres::PgQueryResult};
use uuid::Uuid;

#[async_trait]
pub trait FixedAssetRepository: Send + Sync {
    // ==========================================================
    // ASSET CLASSES
    // ==========================================================

    async fn create_asset_class(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError>;

    async fn list_asset_classes(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AssetClass>, AppError>;

    async fn get_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetClass, AppError>;

    async fn update_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError>;

    async fn delete_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // FIXED ASSETS
    // ==========================================================

    async fn create_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError>;

    async fn capitalize_asset(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
        date: Option<NaiveDate>,
        capitalized_amount: Option<BigDecimal>,
        costs: Option<serde_json::Value>,
    ) -> Result<(), AppError>;

    async fn list_assets(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<FixedAsset>, AppError>;

    async fn get_asset(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<FixedAsset, AppError>;

    async fn update_asset(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError>;

    async fn delete_asset(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;

    // ==========================================================
    // ASSET BOOKS
    // ==========================================================

    async fn create_asset_book(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError>;

    async fn list_asset_books(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetBook>, AppError>;

    async fn get_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetBook, AppError>;

    async fn update_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError>;

    async fn delete_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // CWIP
    // ==========================================================

    async fn create_cwip(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError>;

    async fn list_cwip(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<CapitalWorkInProgress>, AppError>;

    async fn get_cwip(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<CapitalWorkInProgress, AppError>;

    async fn update_cwip(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError>;

    async fn delete_cwip(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError>;

    async fn capitalize_cwip(
        &self,
        pool: &PgPool,
        cwip_uuid: Uuid,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // DEPRECIATION
    // ==========================================================

    async fn run_periodic_depreciation(
        &self,
        pool: &PgPool,
        period_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError>;

    async fn get_depreciation_schedule(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError>;

    async fn post_depreciation_to_gl(
        &self,
        pool: &PgPool,
        depreciation_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    async fn calculate_and_post_depreciation(
        &self,
        pool: &PgPool,
        asset_id: Uuid,
        period_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<AssetDepreciation, AppError>;

    // ==========================================================
    // COMPONENTS
    // ==========================================================

    async fn create_asset_component(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError>;

    async fn list_asset_components(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetComponent>, AppError>;

    async fn get_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetComponent, AppError>;

    async fn update_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError>;

    async fn delete_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // TRANSFERS
    // ==========================================================

    async fn transfer_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetTransfer,
    ) -> Result<AssetTransfer, AppError>;

    async fn asset_transfer_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransfer>, AppError>;

    // ==========================================================
    // MAINTENANCE
    // ==========================================================

    async fn create_maintenance(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError>;

    async fn list_maintenance(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetMaintenance>, AppError>;

    async fn get_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetMaintenance, AppError>;

    async fn update_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError>;

    async fn delete_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // INSURANCE
    // ==========================================================

    async fn create_insurance(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError>;

    async fn list_insurance(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetInsurance>, AppError>;

    async fn get_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetInsurance, AppError>;

    async fn update_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError>;

    async fn delete_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    // ==========================================================
    // REVALUATIONS
    // ==========================================================

    async fn create_revaluation(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetRevaluation,
    ) -> Result<AssetRevaluation, AppError>;

    async fn asset_revaluation_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetRevaluation>, AppError>;

    // ==========================================================
    // DISPOSALS
    // ==========================================================

    async fn dispose_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetDisposal,
    ) -> Result<AssetDisposal, AppError>;

    async fn disposal_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError>;

    // ==========================================================
    // TRANSACTIONS / AUDIT
    // ==========================================================

    async fn asset_transaction_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError>;

    // ==========================================================
    // REPORTS
    // ==========================================================

    async fn fixed_asset_register(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<FixedAsset>, AppError>;

    async fn asset_movements_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError>;

    async fn depreciation_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError>;

    async fn disposed_assets_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError>;
}

pub struct PostgresFixedAssetRepository;

impl PostgresFixedAssetRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FixedAssetRepository for PostgresFixedAssetRepository {
    // =============================================
    // ASSET CLASSES
    // =============================================

    async fn create_asset_class(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError> {
        let asset_class: AssetClass = sqlx::query_as::<_, AssetClass>(
            r#"
        INSERT INTO asset_classes (user_id, class_name, category_type, description)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(&payload.class_name)
        .bind(&payload.category_type)
        .bind(&payload.description)
        .fetch_one(pool)
        .await?;

        Ok(asset_class)
    }

    async fn capitalize_asset(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
        date: Option<NaiveDate>,
        capitalized_amount: Option<BigDecimal>,
        costs: Option<serde_json::Value>, // e.g. {"purchase":1200000, "installation": 80000, ...}
    ) -> Result<(), AppError> {
        let date: NaiveDate = date.unwrap_or_else(|| chrono::Local::now().date_naive());

        sqlx::query(r#"SELECT * FROM fixedassets.capitalize_asset($1, $2, NULL, $3, $4, $5)"#)
            .bind(user_id)
            .bind(asset_uuid)
            .bind(capitalized_amount)
            .bind(date)
            .bind(costs)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn list_asset_classes(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AssetClass>, AppError> {
        let asset_classes: Vec<AssetClass> = sqlx::query_as::<_, AssetClass>(
            "SELECT * FROM asset_classes WHERE user_id = $1 ORDER BY class_name",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(asset_classes)
    }

    async fn get_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetClass, AppError> {
        let asset_class: AssetClass = sqlx::query_as::<_, AssetClass>(
            "SELECT * FROM asset_classes WHERE class_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Asset class not found".into()))?;

        Ok(asset_class)
    }

    async fn update_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError> {
        let asset_class: AssetClass = sqlx::query_as::<_, AssetClass>(
            r#"
        UPDATE asset_classes
        SET class_name = $3, category_type = $4, description = $5
        WHERE class_id = $1 AND user_id = $2
        RETURNING *
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.class_name)
        .bind(&payload.category_type)
        .bind(&payload.description)
        .fetch_one(pool)
        .await?;

        Ok(asset_class)
    }

    async fn delete_asset_class(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res: PgQueryResult =
            sqlx::query("DELETE FROM asset_classes WHERE class_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Asset class not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // FIXED ASSETS
    // ==========================================================
    async fn create_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError> {
        let asset: FixedAsset = sqlx::query_as::<_, FixedAsset>(
            r#"
        INSERT INTO fixed_assets (
            user_id, asset_code, asset_name, description, class_id, location,
            department, custodian_id, tax_class, acquisition_date, supplier_id, po_reference,
            original_cost, capitalized_amount, is_capitalized, capitalization_date,
            financing_method, useful_life_years, residual_value, depreciation_method,
            depreciation_rate, depreciation_start_date, status, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(&payload.asset_code)
        .bind(&payload.asset_name)
        .bind(&payload.description)
        .bind(payload.class_id)
        .bind(&payload.location)
        .bind(&payload.department)
        .bind(payload.custodian_id)
        .bind(&payload.tax_class)
        .bind(payload.acquisition_date)
        .bind(payload.supplier_id)
        .bind(&payload.po_reference)
        .bind(&payload.original_cost)
        .bind(&payload.capitalized_amount)
        .bind(payload.is_capitalized)
        .bind(payload.capitalization_date)
        .bind(&payload.financing_method)
        .bind(payload.useful_life_years)
        .bind(&payload.residual_value)
        .bind(&payload.depreciation_method)
        .bind(&payload.depreciation_rate)
        .bind(payload.depreciation_start_date)
        .bind(&payload.status)
        .bind(user_id) // created_by
        .fetch_one(pool)
        .await?;

        Ok(asset)
    }

    async fn list_assets(&self, pool: &PgPool, user_id: Uuid) -> Result<Vec<FixedAsset>, AppError> {
        let assets: Vec<FixedAsset> = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE user_id = $1 ORDER BY asset_code",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    async fn get_asset(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<FixedAsset, AppError> {
        let asset: FixedAsset = sqlx::query_as::<_, FixedAsset>(
            "SELECT * FROM fixed_assets WHERE asset_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Asset not found".into()))?;

        Ok(asset)
    }

    async fn update_asset(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError> {
        let asset: FixedAsset = sqlx::query_as::<_, FixedAsset>(
            r#"
        UPDATE fixed_assets
        SET asset_name = $3, description = $4, class_id = $5, location = $6,
            department = $7, custodian_id = $8, acquisition_date = $9, tax_class = $10,
            original_cost = $11, capitalized_amount = $12, financing_method = $13,
            useful_life_years = $14, residual_value = $15, depreciation_method = $16,
            depreciation_rate = $17, status = $18
        WHERE asset_id = $1 AND user_id = $2
        RETURNING *
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.asset_name)
        .bind(&payload.description)
        .bind(payload.class_id)
        .bind(&payload.location)
        .bind(&payload.department)
        .bind(payload.custodian_id)
        .bind(payload.acquisition_date)
        .bind(&payload.tax_class)
        .bind(&payload.original_cost)
        .bind(&payload.capitalized_amount)
        .bind(&payload.financing_method)
        .bind(payload.useful_life_years)
        .bind(&payload.residual_value)
        .bind(&payload.depreciation_method)
        .bind(&payload.depreciation_rate)
        .bind(&payload.status)
        .fetch_one(pool)
        .await?;

        Ok(asset)
    }

    async fn delete_asset(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let res: PgQueryResult =
            sqlx::query("DELETE FROM fixed_assets WHERE asset_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Asset not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // ASSET BOOKS
    // ==========================================================

    async fn create_asset_book(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError> {
        let book: AssetBook = sqlx::query_as::<_, AssetBook>(
            r#"
        INSERT INTO asset_books (user_id, asset_id, book_type, useful_life_years,
            depreciation_method, depreciation_rate, residual_value)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(&payload.book_type)
        .bind(payload.useful_life_years)
        .bind(&payload.depreciation_method)
        .bind(&payload.depreciation_rate)
        .bind(&payload.residual_value)
        .fetch_one(pool)
        .await?;

        Ok(book)
    }

    async fn list_asset_books(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetBook>, AppError> {
        let books: Vec<AssetBook> = sqlx::query_as::<_, AssetBook>(
            "SELECT * FROM asset_books WHERE asset_id = $1 AND user_id = $2",
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(books)
    }

    async fn get_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetBook, AppError> {
        let book: AssetBook = sqlx::query_as::<_, AssetBook>(
            "SELECT * FROM asset_books WHERE book_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Asset book not found".into()))?;

        Ok(book)
    }

    async fn update_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError> {
        let book: AssetBook = sqlx::query_as::<_, AssetBook>(
            r#"
        UPDATE asset_books
        SET book_type = $3, useful_life_years = $4, depreciation_method = $5,
            depreciation_rate = $6, residual_value = $7
        WHERE book_id = $1 AND user_id = $2
        RETURNING *
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.book_type)
        .bind(payload.useful_life_years)
        .bind(&payload.depreciation_method)
        .bind(&payload.depreciation_rate)
        .bind(&payload.residual_value)
        .fetch_one(pool)
        .await?;

        Ok(book)
    }

    async fn delete_asset_book(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res: PgQueryResult =
            sqlx::query("DELETE FROM asset_books WHERE book_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Asset book not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // CWIP
    // ==========================================================

    async fn create_cwip(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError> {
        let cwip: CapitalWorkInProgress = sqlx::query_as::<_, CapitalWorkInProgress>(
            r#"
        INSERT INTO asset_cwip (user_id, project_name, total_accumulated_cost,
            start_date, expected_completion_date, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(&payload.project_name)
        .bind(&payload.total_accumulated_cost)
        .bind(payload.start_date)
        .bind(payload.expected_completion_date)
        .bind(&payload.status)
        .fetch_one(pool)
        .await?;

        Ok(cwip)
    }

    async fn list_cwip(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<CapitalWorkInProgress>, AppError> {
        let cwips: Vec<CapitalWorkInProgress> = sqlx::query_as::<_, CapitalWorkInProgress>(
            "SELECT * FROM asset_cwip WHERE user_id = $1 ORDER BY start_date DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(cwips)
    }

    async fn get_cwip(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<CapitalWorkInProgress, AppError> {
        let cwip: CapitalWorkInProgress = sqlx::query_as::<_, CapitalWorkInProgress>(
            "SELECT * FROM asset_cwip WHERE cwip_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("CWIP not found".into()))?;

        Ok(cwip)
    }

    async fn update_cwip(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError> {
        let cwip: CapitalWorkInProgress = sqlx::query_as::<_, CapitalWorkInProgress>(
            r#"
            UPDATE asset_cwip
            SET project_name = $3, total_accumulated_cost = $4,
                expected_completion_date = $5, status = $6
            WHERE cwip_id = $1 AND user_id = $2
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.project_name)
        .bind(&payload.total_accumulated_cost)
        .bind(payload.expected_completion_date)
        .bind(&payload.status)
        .fetch_one(pool)
        .await?;

        Ok(cwip)
    }

    async fn delete_cwip(&self, pool: &PgPool, uuid: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let res: PgQueryResult =
            sqlx::query("DELETE FROM asset_cwip WHERE cwip_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("CWIP not found".into()))
        } else {
            Ok(())
        }
    }

    async fn capitalize_cwip(
        &self,
        pool: &PgPool,
        cwip_uuid: Uuid,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(r#"SELECT * FROM fixedassets.capitalize_asset($1, $2, $3)"#)
            .bind(user_id)
            .bind(asset_uuid)
            .bind(cwip_uuid)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ==========================================================
    // DEPRECIATION
    // ==========================================================

    async fn calculate_and_post_depreciation(
        &self,
        pool: &PgPool,
        asset_id: Uuid,
        period_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<AssetDepreciation, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let dep_record: AssetDepreciation = sqlx::query_as::<_, AssetDepreciation>(
            r#"
            SELECT * FROM calculate_depreciation_full($1, $2, $3)
        "#,
        )
        .bind(user_id)
        .bind(asset_id)
        .bind(period_date)
        .fetch_one(&mut *tx)
        .await?;

        // Insert into depreciation table
        let inserted: AssetDepreciation = sqlx::query_as::<_, AssetDepreciation>(
            r#"
            INSERT INTO fixedassets.asset_depreciation (
                user_id, asset_id, period_date, depreciation_amount, accumulated_depreciation,
                financial_depreciation, accumulated_tax_depreciation, nbv, nbv_financial, twdv,
                posted_to_gl
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, FALSE)
        "#,
        )
        .bind(user_id)
        .bind(asset_id)
        .bind(period_date)
        .bind(dep_record.depreciation_amount)
        .bind(dep_record.accumulated_depreciation)
        .bind(dep_record.financial_depreciation)
        .bind(dep_record.accumulated_tax_depreciation)
        .bind(dep_record.nbv)
        .bind(dep_record.nbv_financial)
        .bind(dep_record.twdv)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(inserted)
    }

    async fn run_periodic_depreciation(
        &self,
        pool: &PgPool,
        period_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        // Run for all active assets
        let deps: Vec<AssetDepreciation> = sqlx::query_as::<_, AssetDepreciation>(
            r#"
            WITH active_assets AS (
                SELECT asset_id FROM fixedassets.fixed_assets
                WHERE user_id = $1 AND status = 'In_Use' AND is_capitalized
            )
            SELECT * FROM active_assets a
            CROSS JOIN LATERAL fixedassets.calculate_depreciation_full($1, a.asset_id, $2) d
            "#,
        )
        .bind(user_id)
        .bind(period_date)
        .fetch_all(pool)
        .await?;

        Ok(deps)
    }

    async fn get_depreciation_schedule(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        let schedule = sqlx::query_as::<_, AssetDepreciation>(
            r#"
        SELECT * FROM asset_depreciation
        WHERE asset_id = $1 AND user_id = $2
        ORDER BY period_date ASC
        "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(schedule)
    }

    async fn post_depreciation_to_gl(
        &self,
        pool: &PgPool,
        depreciation_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res = sqlx::query(
            r#"
        UPDATE asset_depreciation
        SET posted_to_gl = TRUE
        WHERE dep_id = $1 AND user_id = $2
        "#,
        )
        .bind(depreciation_uuid)
        .bind(user_id)
        .execute(pool)
        .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Depreciation record not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // COMPONENTS
    // ==========================================================

    async fn create_asset_component(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError> {
        let component = sqlx::query_as::<_, AssetComponent>(
            r#"
            INSERT INTO asset_components (
                user_id, asset_id, component_name, cost, useful_life_years, depreciation_start_date
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(&payload.component_name)
        .bind(&payload.cost)
        .bind(payload.useful_life_years)
        .bind(payload.depreciation_start_date)
        .fetch_one(pool)
        .await?;

        Ok(component)
    }

    async fn list_asset_components(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetComponent>, AppError> {
        let components = sqlx::query_as::<_, AssetComponent>(
            "SELECT * FROM asset_components WHERE asset_id = $1 AND user_id = $2",
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(components)
    }

    async fn get_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetComponent, AppError> {
        let component = sqlx::query_as::<_, AssetComponent>(
            "SELECT * FROM asset_components WHERE component_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Component not found".into()))?;

        Ok(component)
    }

    async fn update_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError> {
        let component = sqlx::query_as::<_, AssetComponent>(
            r#"
            UPDATE asset_components
            SET component_name = $3, cost = $4, useful_life_years = $5,
                depreciation_start_date = $6
            WHERE component_id = $1 AND user_id = $2
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.component_name)
        .bind(&payload.cost)
        .bind(payload.useful_life_years)
        .bind(payload.depreciation_start_date)
        .fetch_one(pool)
        .await?;

        Ok(component)
    }

    async fn delete_asset_component(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res =
            sqlx::query("DELETE FROM asset_components WHERE component_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Component not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // TRANSFERS
    // ==========================================================

    async fn transfer_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetTransfer,
    ) -> Result<AssetTransfer, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let transfer: AssetTransaction = sqlx::query_as::<_, AssetTransfer>(r#"
            INSERT INTO asset_transactions (
                user_id, asset_id, transaction_type, transaction_date, from_location, to_location, reason
            )
            VALUES ($1, $2, 'Transfer', $3, $4, $5, $6)
            RETURNING *
        "#)
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(payload.transfer_date)
        .bind(&payload.from_location)
        .bind(&payload.to_location)
        .bind(&payload.reason)
        .fetch_one(&mut *tx)
        .await?;

        // Also update current location in fixed_assets
        sqlx::query("UPDATE fixed_assets SET location = $1 WHERE asset_id = $2 AND user_id = $3")
            .bind(&payload.to_location)
            .bind(payload.asset_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(transfer)
    }

    async fn asset_transfer_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransfer>, AppError> {
        let history = sqlx::query_as::<_, AssetTransfer>(
            r#"
        SELECT * FROM asset_transactions
        WHERE asset_id = $1 AND user_id = $2 AND transaction_type = 'Transfer'
        ORDER BY transaction_date DESC
        "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    // ==========================================================
    // MAINTENANCE
    // ==========================================================

    async fn create_maintenance(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError> {
        let maintenance = sqlx::query_as::<_, AssetMaintenance>(
            r#"
        INSERT INTO asset_maintenance (user_id, asset_id, maintenance_type,
            description, cost, maintenance_date, next_due_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(&payload.maintenance_type)
        .bind(&payload.description)
        .bind(&payload.cost)
        .bind(payload.maintenance_date)
        .bind(payload.next_due_date)
        .fetch_one(pool)
        .await?;

        Ok(maintenance)
    }

    async fn list_maintenance(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetMaintenance>, AppError> {
        let records = sqlx::query_as::<_, AssetMaintenance>(
        "SELECT * FROM asset_maintenance WHERE asset_id = $1 AND user_id = $2 ORDER BY maintenance_date DESC"
    )
    .bind(asset_uuid)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

        Ok(records)
    }

    // ==========================================================
    // MAINTENANCE - Remaining Functions
    // ==========================================================

    async fn get_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetMaintenance, AppError> {
        let maintenance = sqlx::query_as::<_, AssetMaintenance>(
            r#"
        SELECT * FROM asset_maintenance WHERE maintenance_id = $1 AND user_id = $2
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Maintenance record not found".into()))?;

        Ok(maintenance)
    }

    async fn update_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError> {
        let maintenance = sqlx::query_as::<_, AssetMaintenance>(
            r#"
        UPDATE asset_maintenance
        SET maintenance_type = $3, description = $4, cost = $5,
            maintenance_date = $6, next_due_date = $7
        WHERE maintenance_id = $1 AND user_id = $2
        RETURNING *
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.maintenance_type)
        .bind(&payload.description)
        .bind(&payload.cost)
        .bind(payload.maintenance_date)
        .bind(payload.next_due_date)
        .fetch_one(pool)
        .await?;

        Ok(maintenance)
    }

    async fn delete_maintenance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res =
            sqlx::query("DELETE FROM asset_maintenance WHERE maintenance_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Maintenance record not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // INSURANCE
    // ==========================================================

    async fn create_insurance(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError> {
        let insurance = sqlx::query_as::<_, AssetInsurance>(
            r#"
        INSERT INTO asset_insurance (
            user_id, asset_id, policy_number, insurer, insured_amount,
            start_date, expiry_date, premium
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(&payload.policy_number)
        .bind(&payload.insurer)
        .bind(&payload.insured_amount)
        .bind(payload.start_date)
        .bind(payload.expiry_date)
        .bind(&payload.premium)
        .fetch_one(pool)
        .await?;

        Ok(insurance)
    }

    async fn list_insurance(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetInsurance>, AppError> {
        let insurances = sqlx::query_as::<_, AssetInsurance>(
            r#"
        SELECT * FROM asset_insurance
        WHERE asset_id = $1 AND user_id = $2
        ORDER BY expiry_date DESC
        "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(insurances)
    }

    async fn get_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetInsurance, AppError> {
        let insurance = sqlx::query_as::<_, AssetInsurance>(
            "SELECT * FROM asset_insurance WHERE insurance_id = $1 AND user_id = $2",
        )
        .bind(uuid)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Insurance record not found".into()))?;

        Ok(insurance)
    }

    async fn update_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError> {
        let insurance = sqlx::query_as::<_, AssetInsurance>(
            r#"
        UPDATE asset_insurance
        SET policy_number = $3, insurer = $4, insured_amount = $5,
            start_date = $6, expiry_date = $7, premium = $8
        WHERE insurance_id = $1 AND user_id = $2
        RETURNING *
        "#,
        )
        .bind(uuid)
        .bind(user_id)
        .bind(&payload.policy_number)
        .bind(&payload.insurer)
        .bind(&payload.insured_amount)
        .bind(payload.start_date)
        .bind(payload.expiry_date)
        .bind(&payload.premium)
        .fetch_one(pool)
        .await?;

        Ok(insurance)
    }

    async fn delete_insurance(
        &self,
        pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let res =
            sqlx::query("DELETE FROM asset_insurance WHERE insurance_id = $1 AND user_id = $2")
                .bind(uuid)
                .bind(user_id)
                .execute(pool)
                .await?;

        if res.rows_affected() == 0 {
            Err(AppError::NotFound("Insurance record not found".into()))
        } else {
            Ok(())
        }
    }

    // ==========================================================
    // REVALUATIONS
    // ==========================================================

    async fn create_revaluation(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetRevaluation,
    ) -> Result<AssetRevaluation, AppError> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let reval: AssetTransaction = sqlx::query_as::<_, AssetRevaluation>(
            r#"
            INSERT INTO fixedassets.asset_transactions (
                user_id, asset_id, transaction_type, transaction_date,
                amount, reason
            )
            VALUES ($1, $2, 'Revaluation', $3, $4, $5)
            RETURNING *
        "#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(payload.revaluation_date)
        .bind(&payload.amount)
        .bind(&payload.reason)
        .fetch_one(&mut *tx)
        .await?;

        // Optional: Update asset's revalued_amount / NBV if using revaluation model
        // sqlx::query!("UPDATE fixed_assets SET ...").execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(reval)
    }

    async fn asset_revaluation_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetRevaluation>, AppError> {
        let history = sqlx::query_as::<_, AssetRevaluation>(
            r#"
        SELECT * FROM asset_transactions
        WHERE asset_id = $1
          AND user_id = $2
          AND transaction_type = 'Revaluation'
        ORDER BY transaction_date DESC
        "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    // ==========================================================
    // HISTORY & AUDIT
    // ==========================================================

    async fn asset_transaction_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError> {
        let history: Vec<AssetTransaction> = sqlx::query_as::<_, AssetTransaction>(
            r#"
                SELECT * FROM fixedassets.asset_transactions
                WHERE asset_id = $1 AND user_id = $2
                ORDER BY transaction_date DESC, created_at DESC
            "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(history)
    }

    async fn dispose_asset(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetDisposal,
    ) -> Result<AssetDisposal, AppError> {
        let disposal: AssetTransaction = sqlx::query_as::<_, AssetDisposal>(
            r#"SELECT * FROM fixedassets.dispose_asset_advanced($1, $2, $3, $4, $5)"#,
        )
        .bind(user_id)
        .bind(payload.asset_id)
        .bind(payload.disposal_date)
        .bind(&payload.proceeds)
        .bind(&payload.reason)
        .fetch_one(pool)
        .await?;

        Ok(disposal)
    }

    async fn disposal_history(
        &self,
        pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError> {
        let disposal: Vec<AssetTransaction> = sqlx::query_as::<_, AssetDisposal>(
            r#"
            SELECT * FROM asset_transactions
            WHERE asset_id = $1
                AND user_id = $2
                AND transaction_type = 'Disposal'
            ORDER BY transaction_date DESC, created_at DESC
            RETURNING *
        "#,
        )
        .bind(asset_uuid)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(disposal)
    }

    // ==========================================================
    // REPORTS
    // ==========================================================

    async fn fixed_asset_register(
        &self,
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<FixedAsset>, AppError> {
        let register: Vec<FixedAsset> = sqlx::query_as::<_, FixedAsset>(
            r#"SELECT * FROM fixed_assets.vw_fixed_asset_register WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(register)
    }

    async fn asset_movements_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError> {
        let movements = sqlx::query_as::<_, AssetTransaction>(
            r#"
        SELECT * FROM asset_transactions
        WHERE user_id = $1
          AND transaction_date BETWEEN $2 AND $3
        ORDER BY transaction_date DESC
        "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;

        Ok(movements)
    }

    async fn depreciation_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        let report = sqlx::query_as::<_, AssetDepreciation>(
            r#"
            SELECT * FROM fixedassets.vw_depreciation_schedule
                WHERE user_id = $1
                AND period_date BETWEEN $2
                AND $3 ORDER BY period_date
            "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;

        Ok(report)
    }

    async fn disposed_assets_report(
        &self,
        pool: &PgPool,
        from_date: NaiveDate,
        to_date: NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError> {
        let disposed = sqlx::query_as::<_, AssetDisposal>(
            r#"
        SELECT * FROM asset_transactions
        WHERE user_id = $1
          AND transaction_type = 'Disposal'
          AND transaction_date BETWEEN $2 AND $3
        "#,
        )
        .bind(user_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(pool)
        .await?;

        Ok(disposed)
    }
}
