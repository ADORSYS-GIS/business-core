use business_core_db::models::product::product::{ProductIdxModel, ProductModel};
use crate::utils::TryFromRow;
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct ProductRepositoryImpl {
    pub executor: Executor,
    pub product_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<ProductIdxModel>>>,
}

impl ProductRepositoryImpl {
    pub fn new(
        executor: Executor,
        product_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<ProductIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            product_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                product_idx_cache,
            ))),
        }
    }

    pub async fn load_all_product_idx(
        executor: &Executor,
    ) -> Result<Vec<ProductIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM product_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.fetch_all(&mut **transaction).await?
            } else {
                return Err(sqlx::Error::PoolTimedOut);
            }
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push(ProductIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for ProductModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ProductModel {
            id: row.get("id"),
            name: row.get("name"),
            product_type: row.get("product_type"),
            minimum_balance: row.get("minimum_balance"),
            maximum_balance: row.try_get("maximum_balance").ok(),
            overdraft_allowed: row.get("overdraft_allowed"),
            overdraft_limit: row.try_get("overdraft_limit").ok(),
            interest_calculation_method: row.get("interest_calculation_method"),
            interest_posting_frequency: row.get("interest_posting_frequency"),
            dormancy_threshold_days: row.get("dormancy_threshold_days"),
            minimum_opening_balance: row.get("minimum_opening_balance"),
            closure_fee: row.get("closure_fee"),
            maintenance_fee: row.try_get("maintenance_fee").ok(),
            maintenance_fee_frequency: row.get("maintenance_fee_frequency"),
            default_dormancy_days: row.try_get("default_dormancy_days").ok(),
            default_overdraft_limit: row.try_get("default_overdraft_limit").ok(),
            per_transaction_limit: row.try_get("per_transaction_limit").ok(),
            daily_transaction_limit: row.try_get("daily_transaction_limit").ok(),
            weekly_transaction_limit: row.try_get("weekly_transaction_limit").ok(),
            monthly_transaction_limit: row.try_get("monthly_transaction_limit").ok(),
            overdraft_interest_rate: row.try_get("overdraft_interest_rate").ok(),
            accrual_frequency: row.get("accrual_frequency"),
            interest_rate_tier_1: row.try_get("interest_rate_tier_1").ok(),
            interest_rate_tier_2: row.try_get("interest_rate_tier_2").ok(),
            interest_rate_tier_3: row.try_get("interest_rate_tier_3").ok(),
            interest_rate_tier_4: row.try_get("interest_rate_tier_4").ok(),
            interest_rate_tier_5: row.try_get("interest_rate_tier_5").ok(),
            account_gl_mapping: row.get("account_gl_mapping"),
            fee_type_gl_mapping: row.get("fee_type_gl_mapping"),
            is_active: row.get("is_active"),
            valid_from: row.get("valid_from"),
            valid_to: row.try_get("valid_to").ok(),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for ProductIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ProductIdxModel {
            id: row.get("id"),
            name: row.get("name"),
            product_type: row.get("product_type"),
            minimum_balance: row.get("minimum_balance"),
            maximum_balance: row.try_get("maximum_balance").ok(),
            overdraft_allowed: row.get("overdraft_allowed"),
            overdraft_limit: row.try_get("overdraft_limit").ok(),
            interest_calculation_method: row.get("interest_calculation_method"),
            interest_posting_frequency: row.get("interest_posting_frequency"),
            dormancy_threshold_days: row.get("dormancy_threshold_days"),
            minimum_opening_balance: row.get("minimum_opening_balance"),
            closure_fee: row.get("closure_fee"),
            maintenance_fee: row.try_get("maintenance_fee").ok(),
            maintenance_fee_frequency: row.get("maintenance_fee_frequency"),
            default_dormancy_days: row.try_get("default_dormancy_days").ok(),
            default_overdraft_limit: row.try_get("default_overdraft_limit").ok(),
            per_transaction_limit: row.try_get("per_transaction_limit").ok(),
            daily_transaction_limit: row.try_get("daily_transaction_limit").ok(),
            weekly_transaction_limit: row.try_get("weekly_transaction_limit").ok(),
            monthly_transaction_limit: row.try_get("monthly_transaction_limit").ok(),
            overdraft_interest_rate: row.try_get("overdraft_interest_rate").ok(),
            accrual_frequency: row.get("accrual_frequency"),
            interest_rate_tier_1: row.try_get("interest_rate_tier_1").ok(),
            interest_rate_tier_2: row.try_get("interest_rate_tier_2").ok(),
            interest_rate_tier_3: row.try_get("interest_rate_tier_3").ok(),
            interest_rate_tier_4: row.try_get("interest_rate_tier_4").ok(),
            interest_rate_tier_5: row.try_get("interest_rate_tier_5").ok(),
            account_gl_mapping: row.get("account_gl_mapping"),
            fee_type_gl_mapping: row.get("fee_type_gl_mapping"),
            is_active: row.get("is_active"),
            valid_from: row.get("valid_from"),
            valid_to: row.try_get("valid_to").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for ProductRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.product_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.product_idx_cache.read().await.on_rollback().await
    }
}