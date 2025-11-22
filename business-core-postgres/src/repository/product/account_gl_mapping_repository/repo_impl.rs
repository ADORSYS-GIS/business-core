use business_core_db::models::product::account_gl_mapping::{AccountGlMappingIdxModel, AccountGlMappingModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct AccountGlMappingRepositoryImpl {
    pub executor: Executor,
    pub account_gl_mapping_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<AccountGlMappingIdxModel>>>,
}

impl AccountGlMappingRepositoryImpl {
    pub fn new(
        executor: Executor,
        account_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<AccountGlMappingIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            account_gl_mapping_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                account_gl_mapping_idx_cache,
            ))),
        }
    }

    pub async fn load_all_account_gl_mapping_idx(
        executor: &Executor,
    ) -> Result<Vec<AccountGlMappingIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM account_gl_mapping_idx");
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
            idx_models.push(AccountGlMappingIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for AccountGlMappingModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(AccountGlMappingModel {
            id: row.get("id"),
            customer_account_code: get_heapless_string(row, "customer_account_code")?,
            overdraft_code: get_optional_heapless_string(row, "overdraft_code")?,
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for AccountGlMappingIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(AccountGlMappingIdxModel {
            id: row.get("id"),
            customer_account_code: get_heapless_string(row, "customer_account_code")?,
            overdraft_code: get_optional_heapless_string(row, "overdraft_code")?,
        })
    }
}

#[async_trait]
impl TransactionAware for AccountGlMappingRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.account_gl_mapping_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.account_gl_mapping_idx_cache.read().await.on_rollback().await
    }
}