use business_core_db::models::person::location::{LocationIdxModel, LocationModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct LocationRepositoryImpl {
    pub executor: Executor,
    pub location_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<LocationIdxModel>>>,
}

impl LocationRepositoryImpl {
    pub fn new(
        executor: Executor,
        location_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<LocationIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            location_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                location_idx_cache,
            ))),
        }
    }

    pub async fn load_all_location_idx(
        executor: &Executor,
    ) -> Result<Vec<LocationIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM location_idx");
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
            idx_models.push(LocationIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for LocationModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(LocationModel {
            id: row.get("id"),
            street_line1: get_heapless_string(row, "street_line1")?,
            street_line2: get_optional_heapless_string(row, "street_line2")?,
            street_line3: get_optional_heapless_string(row, "street_line3")?,
            street_line4: get_optional_heapless_string(row, "street_line4")?,
            locality_id: row.get("locality_id"),
            postal_code: get_optional_heapless_string(row, "postal_code")?,
            latitude: row.try_get("latitude").ok(),
            longitude: row.try_get("longitude").ok(),
            accuracy_meters: row.try_get("accuracy_meters").ok(),
            location_type: row.get("location_type"),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for LocationIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(LocationIdxModel {
            id: row.get("location_id"),
            locality_id: row.get("locality_id"),
        })
    }
}

#[async_trait]
impl TransactionAware for LocationRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.location_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.location_idx_cache.read().await.on_rollback().await
    }
}