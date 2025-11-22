use business_core_db::models::product::account_gl_mapping::{AccountGlMappingIdxModel, AccountGlMappingModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::{TransactionAwareIdxModelCache, TransactionAwareMainModelCache};
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct AccountGlMappingRepositoryImpl {
    pub executor: Executor,
    pub account_gl_mapping_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<AccountGlMappingIdxModel>>>,
    pub account_gl_mapping_cache: Arc<RwLock<TransactionAwareMainModelCache<AccountGlMappingModel>>>,
}

impl AccountGlMappingRepositoryImpl {
    pub fn new(
        executor: Executor,
        account_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<AccountGlMappingIdxModel>>>,
        account_gl_mapping_cache: Arc<ParkingRwLock<postgres_index_cache::MainModelCache<AccountGlMappingModel>>>,
    ) -> Self {
        Self {
            executor,
            account_gl_mapping_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                account_gl_mapping_idx_cache,
            ))),
            account_gl_mapping_cache: Arc::new(RwLock::new(TransactionAwareMainModelCache::new(
                account_gl_mapping_cache,
            ))),
        }
    }

    pub async fn load_all_account_gl_mapping_idx(
        executor: &Executor,
    ) -> Result<Vec<AccountGlMappingIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM account_gl_mapping_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            let transaction = tx.as_mut().expect("Transaction has been consumed");
            query.fetch_all(&mut **transaction).await?
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push(AccountGlMappingIdxModel::try_from_row(&row).unwrap());
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
            hash: row.get("hash"),
            audit_log_id: row.get("audit_log_id"),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
        })
    }
}

impl TryFromRow<PgRow> for AccountGlMappingIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(AccountGlMappingIdxModel {
            id: row.get("account_gl_mapping_id"),
            customer_account_code_hash: row.get("customer_account_code_hash"),
        })
    }
}

#[async_trait]
impl TransactionAware for AccountGlMappingRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.account_gl_mapping_idx_cache.read().await.on_commit().await?;
        self.account_gl_mapping_cache.read().await.on_commit().await?;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.account_gl_mapping_idx_cache.read().await.on_rollback().await?;
        self.account_gl_mapping_cache.read().await.on_rollback().await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context_and_listen, create_test_audit_log};
    use business_core_db::models::index_aware::IndexAware;
    use tokio::time::{sleep, Duration};
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;

    #[tokio::test]
    async fn test_account_gl_mapping_insert_triggers_cache_notifications() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test entity
        let test_entity = create_test_account_gl_mapping("12345");
        let entity_idx = test_entity.to_index();

        // Give listener time to start
        sleep(Duration::from_millis(2000)).await;

        // Create audit log
        let audit_log = create_test_audit_log();
        sqlx::query("INSERT INTO audit_log (id, updated_at, updated_by_person_id) VALUES ($1, $2, $3)")
            .bind(audit_log.id)
            .bind(audit_log.updated_at)
            .bind(audit_log.updated_by_person_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert audit log");

        // Prepare entity with hash
        let mut test_entity_for_hashing = test_entity.clone();
        test_entity_for_hashing.hash = 0;
        test_entity_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash = business_core_db::utils::hash_as_i64(&test_entity_for_hashing).unwrap();
        
        let final_entity = AccountGlMappingModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_entity
        };

        // Insert the entity record directly into database
        sqlx::query("INSERT INTO account_gl_mapping (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(final_entity.id)
            .bind(&final_entity.customer_account_code)
            .bind(&final_entity.overdraft_code)
            .bind(final_entity.hash)
            .bind(final_entity.audit_log_id)
            .bind(final_entity.antecedent_hash)
            .bind(final_entity.antecedent_audit_log_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert account_gl_mapping");

        // Insert the index record directly into database
        sqlx::query("INSERT INTO account_gl_mapping_idx (account_gl_mapping_id, customer_account_code_hash) VALUES ($1, $2)")
            .bind(entity_idx.id)
            .bind(entity_idx.customer_account_code_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert account_gl_mapping index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let entity_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Verify the INDEX cache was updated
        let idx_cache = entity_repo.account_gl_mapping_idx_cache.read().await;
        assert!(
            idx_cache.contains_primary(&entity_idx.id),
            "AccountGlMapping should be in index cache after insert"
        );
        drop(idx_cache);

        // Verify the MAIN cache was updated
        let main_cache = entity_repo.account_gl_mapping_cache.read().await;
        assert!(
            main_cache.contains(&final_entity.id),
            "AccountGlMapping should be in main cache after insert"
        );
        drop(main_cache);

        // Delete the record from database
        sqlx::query("DELETE FROM account_gl_mapping WHERE id = $1")
            .bind(final_entity.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete account_gl_mapping");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify removed from both caches
        let idx_cache = entity_repo.account_gl_mapping_idx_cache.read().await;
        assert!(
            !idx_cache.contains_primary(&entity_idx.id),
            "AccountGlMapping should be removed from index cache after delete"
        );
        drop(idx_cache);

        let main_cache = entity_repo.account_gl_mapping_cache.read().await;
        assert!(
            !main_cache.contains(&final_entity.id),
            "AccountGlMapping should be removed from main cache after delete"
        );

        Ok(())
    }
}