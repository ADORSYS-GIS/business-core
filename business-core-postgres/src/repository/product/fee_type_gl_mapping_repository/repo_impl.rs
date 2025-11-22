use business_core_db::models::product::fee_type_gl_mapping::{FeeTypeGlMappingIdxModel, FeeTypeGlMappingModel};
use crate::utils::{get_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct FeeTypeGlMappingRepositoryImpl {
    pub executor: Executor,
    pub fee_type_gl_mapping_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<FeeTypeGlMappingIdxModel>>>,
}

impl FeeTypeGlMappingRepositoryImpl {
    pub fn new(
        executor: Executor,
        fee_type_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<FeeTypeGlMappingIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            fee_type_gl_mapping_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                fee_type_gl_mapping_idx_cache,
            ))),
        }
    }

    pub async fn load_all_fee_type_gl_mapping_idx(
        executor: &Executor,
    ) -> Result<Vec<FeeTypeGlMappingIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM fee_type_gl_mapping_idx");
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
            idx_models.push(FeeTypeGlMappingIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for FeeTypeGlMappingModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(FeeTypeGlMappingModel {
            id: row.get("id"),
            fee_type: row.get("fee_type"),
            gl_code: get_heapless_string(row, "gl_code")?,
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for FeeTypeGlMappingIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(FeeTypeGlMappingIdxModel {
            id: row.get("id"),
            fee_type: row.get("fee_type"),
            gl_code: get_heapless_string(row, "gl_code")?,
        })
    }
}

#[async_trait]
impl TransactionAware for FeeTypeGlMappingRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.fee_type_gl_mapping_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.fee_type_gl_mapping_idx_cache.read().await.on_rollback().await
    }
}