use business_core_db::models::calendar::weekend_days::{WeekendDaysIdxModel, WeekendDaysModel};
use crate::utils::TryFromRow;
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::{TransactionAwareIdxModelCache, TransactionAwareMainModelCache};
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct WeekendDaysRepositoryImpl {
    pub executor: Executor,
    pub weekend_days_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<WeekendDaysIdxModel>>>,
    pub weekend_days_cache: Arc<RwLock<TransactionAwareMainModelCache<WeekendDaysModel>>>,
}

impl WeekendDaysRepositoryImpl {
    pub fn new(
        executor: Executor,
        weekend_days_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<WeekendDaysIdxModel>>>,
        weekend_days_cache: Arc<ParkingRwLock<postgres_index_cache::MainModelCache<WeekendDaysModel>>>,
    ) -> Self {
        Self {
            executor,
            weekend_days_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                weekend_days_idx_cache,
            ))),
            weekend_days_cache: Arc::new(RwLock::new(TransactionAwareMainModelCache::new(
                weekend_days_cache,
            ))),
        }
    }

    pub async fn load_all_weekend_days_idx(
        executor: &Executor,
    ) -> Result<Vec<WeekendDaysIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM calendar_weekend_days_idx");
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
            idx_models.push(WeekendDaysIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for WeekendDaysRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.weekend_days_idx_cache.read().await.on_commit().await?;
        self.weekend_days_cache.read().await.on_commit().await?;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.weekend_days_idx_cache.read().await.on_rollback().await?;
        self.weekend_days_cache.read().await.on_rollback().await?;
        Ok(())
    }
}

impl TryFromRow<PgRow> for WeekendDaysModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(WeekendDaysModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            weekend_day_01: row.get("weekend_day_01"),
            weekend_day_02: row.get("weekend_day_02"),
            weekend_day_03: row.get("weekend_day_03"),
            weekend_day_04: row.get("weekend_day_04"),
            weekend_day_05: row.get("weekend_day_05"),
            weekend_day_06: row.get("weekend_day_06"),
            weekend_day_07: row.get("weekend_day_07"),
            effective_date: row.get("effective_date"),
            expiry_date: row.get("expiry_date"),
        })
    }
}

impl TryFromRow<PgRow> for WeekendDaysIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(WeekendDaysIdxModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
        })
    }
}