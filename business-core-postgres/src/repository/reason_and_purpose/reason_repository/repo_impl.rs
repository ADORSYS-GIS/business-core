use business_core_db::models::reason_and_purpose::reason::{ReasonIdxModel, ReasonModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct ReasonRepositoryImpl {
    pub executor: Executor,
    pub reason_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<ReasonIdxModel>>>,
}

impl ReasonRepositoryImpl {
    pub fn new(
        executor: Executor,
        reason_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<ReasonIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            reason_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                reason_idx_cache,
            ))),
        }
    }

    pub async fn load_all_reason_idx(
        executor: &Executor,
    ) -> Result<Vec<ReasonIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM reason_idx");
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
            idx_models.push(ReasonIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for ReasonRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.reason_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.reason_idx_cache.read().await.on_rollback().await
    }
}

impl TryFromRow<PgRow> for ReasonModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ReasonModel {
            id: row.get("id"),
            code: get_heapless_string(row, "code")?,
            category: row.get("category"),
            context: row.get("context"),
            l1_content: get_optional_heapless_string(row, "l1_content")?,
            l2_content: get_optional_heapless_string(row, "l2_content")?,
            l3_content: get_optional_heapless_string(row, "l3_content")?,
            l1_language_code: get_optional_heapless_string(row, "l1_language_code")?,
            l2_language_code: get_optional_heapless_string(row, "l2_language_code")?,
            l3_language_code: get_optional_heapless_string(row, "l3_language_code")?,
            requires_details: row.get("requires_details"),
            is_active: row.get("is_active"),
            severity: row.get("severity"),
            display_order: row.get("display_order"),
            compliance_metadata: row.get("compliance_metadata"),
        })
    }
}

impl TryFromRow<PgRow> for ReasonIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ReasonIdxModel {
            id: row.get("reason_id"),
            code_hash: row.try_get("code_hash")?,
            category_hash: row.try_get("category_hash")?,
            context_hash: row.try_get("context_hash")?,
            compliance_metadata: row.get("compliance_metadata"),
        })
    }
}