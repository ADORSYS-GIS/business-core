use business_core_db::models::calendar::business_day::{BusinessDayIdxModel, BusinessDayModel};
use crate::utils::{get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::{TransactionAwareIdxModelCache, TransactionAwareMainModelCache};
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct BusinessDayRepositoryImpl {
    pub executor: Executor,
    pub business_day_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<BusinessDayIdxModel>>>,
    pub business_day_cache: Arc<RwLock<TransactionAwareMainModelCache<BusinessDayModel>>>,
}

impl BusinessDayRepositoryImpl {
    pub fn new(
        executor: Executor,
        business_day_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<BusinessDayIdxModel>>>,
        business_day_cache: Arc<ParkingRwLock<postgres_index_cache::MainModelCache<BusinessDayModel>>>,
    ) -> Self {
        Self {
            executor,
            business_day_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                business_day_idx_cache,
            ))),
            business_day_cache: Arc::new(RwLock::new(TransactionAwareMainModelCache::new(
                business_day_cache,
            ))),
        }
    }

    pub async fn load_all_business_day_idx(
        executor: &Executor,
    ) -> Result<Vec<BusinessDayIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM calendar_business_day_idx");
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
            idx_models.push(BusinessDayIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for BusinessDayRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.business_day_idx_cache.read().await.on_commit().await?;
        self.business_day_cache.read().await.on_commit().await?;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.business_day_idx_cache.read().await.on_rollback().await?;
        self.business_day_cache.read().await.on_rollback().await?;
        Ok(())
    }
}

impl TryFromRow<PgRow> for BusinessDayModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(BusinessDayModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            date: row.get("date"),
            weekday: row.get("weekday"),
            is_business_day: row.get("is_business_day"),
            is_weekend: row.get("is_weekend"),
            weekend_day_01: row.get("weekend_day_01"),
            is_holiday: row.get("is_holiday"),
            holiday_name: get_optional_heapless_string(row, "holiday_name")?,
            day_scope: row.get("day_scope"),
        })
    }
}

impl TryFromRow<PgRow> for BusinessDayIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(BusinessDayIdxModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            date_hash: row.get("date_hash"),
        })
    }
}