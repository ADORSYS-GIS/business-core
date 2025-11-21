use business_core_db::models::description::named::{NamedIdxModel, NamedModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct NamedRepositoryImpl {
    pub executor: Executor,
    pub named_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<NamedIdxModel>>>,
}

impl NamedRepositoryImpl {
    pub fn new(
        executor: Executor,
        named_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<NamedIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            named_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                named_idx_cache,
            ))),
        }
    }

    pub async fn load_all_named_idx(
        executor: &Executor,
    ) -> Result<Vec<NamedIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM named_idx");
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
            idx_models.push(NamedIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for NamedModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(NamedModel {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            name_l1: get_heapless_string(row, "name_l1")?,
            name_l2: get_optional_heapless_string(row, "name_l2")?,
            name_l3: get_optional_heapless_string(row, "name_l3")?,
            name_l4: get_optional_heapless_string(row, "name_l4")?,
            description_l1: get_optional_heapless_string(row, "description_l1")?,
            description_l2: get_optional_heapless_string(row, "description_l2")?,
            description_l3: get_optional_heapless_string(row, "description_l3")?,
            description_l4: get_optional_heapless_string(row, "description_l4")?,
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for NamedIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(NamedIdxModel {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
        })
    }
}

#[async_trait]
impl TransactionAware for NamedRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.named_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.named_idx_cache.read().await.on_rollback().await
    }
}