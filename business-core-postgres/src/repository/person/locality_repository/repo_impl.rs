use business_core_db::models::person::locality::{LocalityIdxModel, LocalityModel};
use crate::utils::{get_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct LocalityRepositoryImpl {
    pub executor: Executor,
    pub locality_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<LocalityIdxModel>>>,
}

impl LocalityRepositoryImpl {
    pub fn new(
        executor: Executor,
        locality_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<LocalityIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            locality_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                locality_idx_cache,
            ))),
        }
    }

    pub async fn load_all_locality_idx(
        executor: &Executor,
    ) -> Result<Vec<LocalityIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM locality_idx");
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
            idx_models.push(LocalityIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for LocalityRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.locality_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.locality_idx_cache.read().await.on_rollback().await
    }
}

impl TryFromRow<PgRow> for LocalityModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(LocalityModel {
            id: row.get("id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            code: get_heapless_string(row, "code")?,
            name: row.get("name"),
        })
    }
}

impl TryFromRow<PgRow> for LocalityIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(LocalityIdxModel {
            id: row.get("id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            code_hash: row.try_get("code_hash")?,
        })
    }
}