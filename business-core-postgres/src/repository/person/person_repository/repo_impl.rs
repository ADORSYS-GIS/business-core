use business_core_db::models::person::person::{PersonIdxModel, PersonModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct PersonRepositoryImpl {
    pub executor: Executor,
    pub person_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<PersonIdxModel>>>,
}

impl PersonRepositoryImpl {
    pub fn new(
        executor: Executor,
        person_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<PersonIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            person_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                person_idx_cache,
            ))),
        }
    }

    pub async fn load_all_person_idx(
        executor: &Executor,
    ) -> Result<Vec<PersonIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM person_idx");
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
            idx_models.push(PersonIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for PersonModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(PersonModel {
            id: row.get("id"),
            person_type: row.get("person_type"),
            risk_rating: row.get("risk_rating"),
            status: row.get("status"),
            display_name: get_heapless_string(row, "display_name")?,
            external_identifier: get_optional_heapless_string(row, "external_identifier")?,
            id_type: row.get("id_type"),
            id_number: get_heapless_string(row, "id_number")?,
            entity_reference_count: row.get("entity_reference_count"),
            organization_person_id: row.try_get("organization_person_id").ok(),
            messaging_info1: get_optional_heapless_string(row, "messaging_info1")?,
            messaging_info2: get_optional_heapless_string(row, "messaging_info2")?,
            messaging_info3: get_optional_heapless_string(row, "messaging_info3")?,
            messaging_info4: get_optional_heapless_string(row, "messaging_info4")?,
            messaging_info5: get_optional_heapless_string(row, "messaging_info5")?,
            department: get_optional_heapless_string(row, "department")?,
            location_id: row.try_get("location_id").ok(),
            duplicate_of_person_id: row.try_get("duplicate_of_person_id").ok(),
            last_activity_log: row.try_get("last_activity_log").ok(),
            last_compliance_status: row.try_get("last_compliance_status").ok(),
            last_document: row.try_get("last_document").ok(),
            last_portfolio: row.try_get("last_portfolio").ok(),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

impl TryFromRow<PgRow> for PersonIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(PersonIdxModel {
            id: row.get("id"),
            external_identifier_hash: row.try_get("external_identifier_hash").ok(),
            organization_person_id: row.try_get("organization_person_id").ok(),
            duplicate_of_person_id: row.try_get("duplicate_of_person_id").ok(),
            id_number_hash: row.try_get("id_number_hash").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for PersonRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.person_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.person_idx_cache.read().await.on_rollback().await
    }
}