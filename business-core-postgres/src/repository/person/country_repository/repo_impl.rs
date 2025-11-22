use business_core_db::models::person::country::{CountryIdxModel, CountryModel};
use crate::utils::{get_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct CountryRepositoryImpl {
    pub executor: Executor,
    pub country_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<CountryIdxModel>>>,
}

impl CountryRepositoryImpl {
    pub fn new(
        executor: Executor,
        country_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            country_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                country_idx_cache,
            ))),
        }
    }

    pub async fn load_all_country_idx(
        executor: &Executor,
    ) -> Result<Vec<CountryIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM country_idx");
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
            idx_models.push(CountryIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for CountryRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.country_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.country_idx_cache.read().await.on_rollback().await
    }
}

impl TryFromRow<PgRow> for CountryModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountryModel {
            id: row.get("id"),
            iso2: get_heapless_string(row, "iso2")?,
            name: row.get("name"),
        })
    }
}

impl TryFromRow<PgRow> for CountryIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountryIdxModel {
            id: row.get("country_id"),
            iso2_hash: row.try_get("iso2_hash")?,
        })
    }
}
