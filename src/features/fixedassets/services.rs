// src/routes/vendors/service.rs
use crate::{
    features::fixedassets::repository::FixedAssetRepository, interface::api::errors::AppError,
    models::fixed_assets::*,
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct FixedAssetService<R: FixedAssetRepository> {
    repo: R,
}

impl<R: FixedAssetRepository> FixedAssetService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // Asset Class
    pub async fn new_asset_class(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError> {
        self.repo
            .create_asset_class(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_asset_classes(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<AssetClass>, AppError> {
        self.repo.list_asset_classes(tenant_pool, user_id).await
    }

    pub async fn get_asset_class(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetClass, AppError> {
        self.repo.get_asset_class(tenant_pool, uuid, user_id).await
    }

    pub async fn update_asset_class(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetClass,
    ) -> Result<AssetClass, AppError> {
        self.repo
            .update_asset_class(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn delete_asset_class(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_asset_class(tenant_pool, uuid, user_id)
            .await
    }

    // Asset
    pub async fn new_asset(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError> {
        self.repo.create_asset(tenant_pool, user_id, payload).await
    }

    pub async fn list_asset(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<FixedAsset>, AppError> {
        self.repo.list_assets(tenant_pool, user_id).await
    }

    pub async fn get_asset(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<FixedAsset, AppError> {
        self.repo.get_asset(tenant_pool, uuid, user_id).await
    }

    pub async fn update_asset(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAsset,
    ) -> Result<FixedAsset, AppError> {
        self.repo
            .update_asset(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn delete_asset(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_asset(tenant_pool, uuid, user_id).await
    }

    // Asset Books
    pub async fn new_asset_book(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError> {
        self.repo
            .create_asset_book(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_asset_books(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetBook>, AppError> {
        self.repo
            .list_asset_books(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn get_asset_book(
        &self,
        tenant_pool: &PgPool,
        assetbook_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetBook, AppError> {
        self.repo
            .get_asset_book(tenant_pool, assetbook_uuid, user_id)
            .await
    }

    pub async fn update_asset_book(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetBook,
    ) -> Result<AssetBook, AppError> {
        self.repo
            .update_asset_book(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn delete_asset_book(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_asset_book(tenant_pool, uuid, user_id)
            .await
    }

    // CWIP
    pub async fn new_cwip(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError> {
        self.repo.create_cwip(tenant_pool, user_id, payload).await
    }

    pub async fn list_cwip(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<CapitalWorkInProgress>, AppError> {
        self.repo.list_cwip(tenant_pool, user_id).await
    }

    pub async fn get_cwip(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<CapitalWorkInProgress, AppError> {
        self.repo.get_cwip(tenant_pool, uuid, user_id).await
    }

    pub async fn update_cwip(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
        payload: &CreateCWIP,
    ) -> Result<CapitalWorkInProgress, AppError> {
        self.repo
            .update_cwip(tenant_pool, uuid, user_id, payload)
            .await
    }

    pub async fn capitalize_cwip(
        &self,
        tenant_pool: &PgPool,
        cwip_uuid: Uuid,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .capitalize_cwip(tenant_pool, cwip_uuid, asset_uuid, user_id)
            .await
    }

    pub async fn delete_cwip(
        &self,
        tenant_pool: &PgPool,
        uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_cwip(tenant_pool, uuid, user_id).await
    }

    // Depreciation
    pub async fn compute_depreciation_and_post(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        period: chrono::NaiveDate,
        asset_uuid: Uuid,
    ) -> Result<AssetDepreciation, AppError> {
        self.repo
            .calculate_and_post_depreciation(tenant_pool, asset_uuid, period, user_id)
            .await
    }

    pub async fn run_depreciation(
        &self,
        tenant_pool: &PgPool,
        period_date: chrono::NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        self.repo
            .run_depreciation(tenant_pool, period_date, user_id)
            .await
    }

    pub async fn get_depreciation_schedule(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        self.repo
            .get_depreciation_schedule(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn post_depreciation_to_gl(
        &self,
        tenant_pool: &PgPool,
        depreciation_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .post_depreciation_to_gl(tenant_pool, depreciation_uuid, user_id)
            .await
    }

    // Components
    pub async fn create_asset_component(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError> {
        self.repo
            .create_asset_component(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_asset_components(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetComponent>, AppError> {
        self.repo
            .list_asset_components(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn get_asset_component(
        &self,
        tenant_pool: &PgPool,
        asset_comp_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetComponent, AppError> {
        self.repo
            .get_asset_component(tenant_pool, asset_comp_uuid, user_id)
            .await
    }

    pub async fn update_asset_component(
        &self,
        tenant_pool: &PgPool,
        asset_comp_uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetComponent,
    ) -> Result<AssetComponent, AppError> {
        self.repo
            .update_asset_component(tenant_pool, asset_comp_uuid, user_id, payload)
            .await
    }

    pub async fn delete_asset_component(
        &self,
        tenant_pool: &PgPool,
        asset_comp_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_asset_component(tenant_pool, asset_comp_uuid, user_id)
            .await
    }

    // Transfer asset
    pub async fn transfer_asset(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetTransfer,
    ) -> Result<AssetTransfer, AppError> {
        self.repo
            .transfer_asset(tenant_pool, user_id, payload)
            .await
    }

    pub async fn asset_transfer_history(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransfer>, AppError> {
        self.repo
            .asset_transfer_history(tenant_pool, asset_uuid, user_id)
            .await
    }

    // Maintanance
    pub async fn create_maintenance(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError> {
        self.repo
            .create_maintenance(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_maintenance(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetMaintenance>, AppError> {
        self.repo
            .list_maintenance(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn get_maintenance(
        &self,
        tenant_pool: &PgPool,
        maintanance_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetMaintenance, AppError> {
        self.repo
            .get_maintenance(tenant_pool, maintanance_uuid, user_id)
            .await
    }

    pub async fn update_maintenance(
        &self,
        tenant_pool: &PgPool,
        maintanance_uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetMaintenance,
    ) -> Result<AssetMaintenance, AppError> {
        self.repo
            .update_maintenance(tenant_pool, maintanance_uuid, user_id, payload)
            .await
    }

    pub async fn delete_maintenance(
        &self,
        tenant_pool: &PgPool,
        maintanance_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_maintenance(tenant_pool, maintanance_uuid, user_id)
            .await
    }

    // Insurance
    pub async fn create_insurance(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError> {
        self.repo
            .create_insurance(tenant_pool, user_id, payload)
            .await
    }

    pub async fn list_insurance(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetInsurance>, AppError> {
        self.repo
            .list_insurance(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn get_insurance(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<AssetInsurance, AppError> {
        self.repo
            .get_insurance(tenant_pool, asset_uuid, user_id)
            .await
    }

    pub async fn update_insurance(
        &self,
        tenant_pool: &PgPool,
        insurance_uuid: Uuid,
        user_id: Uuid,
        payload: &CreateAssetInsurance,
    ) -> Result<AssetInsurance, AppError> {
        self.repo
            .update_insurance(tenant_pool, insurance_uuid, user_id, payload)
            .await
    }

    pub async fn delete_insurance(
        &self,
        tenant_pool: &PgPool,
        insurance_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        self.repo
            .delete_insurance(tenant_pool, insurance_uuid, user_id)
            .await
    }

    // Revaluation
    pub async fn create_revaluation(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetRevaluation,
    ) -> Result<AssetRevaluation, AppError> {
        self.repo
            .create_revaluation(tenant_pool, user_id, payload)
            .await
    }

    pub async fn asset_revaluation_history(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetRevaluation>, AppError> {
        self.repo
            .asset_revaluation_history(tenant_pool, asset_uuid, user_id)
            .await
    }

    // Disposal
    pub async fn dispose_asset(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
        payload: &CreateAssetDisposal,
    ) -> Result<AssetDisposal, AppError> {
        self.repo.dispose_asset(tenant_pool, user_id, payload).await
    }

    pub async fn disposal_history(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError> {
        self.repo
            .disposal_history(tenant_pool, asset_uuid, user_id)
            .await
    }

    // Reports
    pub async fn fixed_asset_register(
        &self,
        tenant_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<FixedAsset>, AppError> {
        self.repo.fixed_asset_register(tenant_pool, user_id).await
    }

    pub async fn asset_movements_report(
        &self,
        tenant_pool: &PgPool,
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError> {
        self.repo
            .asset_movements_report(tenant_pool, from_date, to_date, user_id)
            .await
    }

    pub async fn depreciation_report(
        &self,
        tenant_pool: &PgPool,
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDepreciation>, AppError> {
        self.repo
            .depreciation_report(tenant_pool, from_date, to_date, user_id)
            .await
    }

    pub async fn disposed_assets_report(
        &self,
        tenant_pool: &PgPool,
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
        user_id: Uuid,
    ) -> Result<Vec<AssetDisposal>, AppError> {
        self.repo
            .disposed_assets_report(tenant_pool, from_date, to_date, user_id)
            .await
    }

    pub async fn asset_transaction_history(
        &self,
        tenant_pool: &PgPool,
        asset_uuid: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<AssetTransaction>, AppError> {
        self.repo
            .asset_transaction_history(tenant_pool, asset_uuid, user_id)
            .await
    }
}
