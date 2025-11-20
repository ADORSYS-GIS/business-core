use business_core_db::models::person::risk_summary::{RiskSummaryIdxModel, RiskSummaryModel};
use crate::utils::{get_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct RiskSummaryRepositoryImpl {
    pub executor: Executor,
    pub risk_summary_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<RiskSummaryIdxModel>>>,
}

impl RiskSummaryRepositoryImpl {
    pub fn new(
        executor: Executor,
        risk_summary_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<RiskSummaryIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            risk_summary_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                risk_summary_idx_cache,
            ))),
        }
    }

    pub async fn load_all_risk_summary_idx(
        executor: &Executor,
    ) -> Result<Vec<RiskSummaryIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM risk_summary_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or(sqlx::Error::PoolTimedOut)?;
            query.fetch_all(&mut **transaction).await?
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push(RiskSummaryIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for RiskSummaryModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(RiskSummaryModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            current_rating: row.get("current_rating"),
            last_assessment_date: row.get("last_assessment_date"),
            flags_01: get_heapless_string(row, "flags_01")?,
            flags_02: get_heapless_string(row, "flags_02")?,
            flags_03: get_heapless_string(row, "flags_03")?,
            flags_04: get_heapless_string(row, "flags_04")?,
            flags_05: get_heapless_string(row, "flags_05")?,
        })
    }
}

impl TryFromRow<PgRow> for RiskSummaryIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(RiskSummaryIdxModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
        })
    }
}

#[async_trait]
impl TransactionAware for RiskSummaryRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.risk_summary_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.risk_summary_idx_cache.read().await.on_rollback().await
    }
}