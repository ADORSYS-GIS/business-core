use business_core_db::models::person::entity_reference::{EntityReferenceIdxModel, EntityReferenceModel};
use crate::utils::{get_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;
use uuid::Uuid;

pub struct EntityReferenceRepositoryImpl {
    pub executor: Executor,
    pub entity_reference_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<EntityReferenceIdxModel>>>,
}

impl EntityReferenceRepositoryImpl {
    pub fn new(
        executor: Executor,
        entity_reference_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<EntityReferenceIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            entity_reference_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                entity_reference_idx_cache,
            ))),
        }
    }

    pub async fn load_all_entity_reference_idx(
        executor: &Executor,
    ) -> Result<Vec<EntityReferenceIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM entity_reference_idx");
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
            idx_models.push(EntityReferenceIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }

    pub async fn find_ids_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.entity_reference_idx_cache.read().await;
        let items = cache.get_by_uuid_index("person_id", &person_id);
        let result = items.into_iter().map(|item| item.id).collect();
        Ok(result)
    }

    pub async fn find_ids_by_reference_external_id_hash(
        &self,
        reference_external_id_hash: i64,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.entity_reference_idx_cache.read().await;
        let items = cache.get_by_i64_index("reference_external_id_hash", &reference_external_id_hash);
        let result = items.into_iter().map(|item| item.id).collect();
        Ok(result)
    }
}

impl TryFromRow<PgRow> for EntityReferenceModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(EntityReferenceModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            entity_role: row.get("entity_role"),
            reference_external_id: get_heapless_string(row, "reference_external_id")?,
            reference_details: row.try_get("reference_details").ok(),
            related_person_id: row.try_get("related_person_id").ok(),
            start_date: row.try_get("start_date").ok(),
            end_date: row.try_get("end_date").ok(),
            status: row.try_get("status").ok(),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for EntityReferenceIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(EntityReferenceIdxModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            reference_external_id_hash: row.get("reference_external_id_hash"),
        })
    }
}

#[async_trait]
impl TransactionAware for EntityReferenceRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.entity_reference_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.entity_reference_idx_cache.read().await.on_rollback().await
    }
}