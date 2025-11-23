use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    product::fee_type_gl_mapping::FeeTypeGlMappingModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &FeeTypeGlMappingRepositoryImpl,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for FeeTypeGlMappingModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for mut item in items {
                // 1. Create a copy of entity for hashing
                let mut entity_for_hashing = item.clone();
                entity_for_hashing.hash = 0;  // Must be 0 before hashing
                entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing

                // 2. Compute hash
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                // 3. Update original entity with computed hash and new audit_log_id
                item.hash = computed_hash;
                item.audit_log_id = Some(audit_log_id);

                // Execute audit insert
                sqlx::query(
                    r#"
                    INSERT INTO fee_type_gl_mapping_audit
                    (id, fee_type, gl_code, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(item.id)
                .bind(item.fee_type.clone())
                .bind(item.gl_code.as_str())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO fee_type_gl_mapping
                    (id, fee_type, gl_code, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(item.id)
                .bind(item.fee_type.clone())
                .bind(item.gl_code.as_str())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO fee_type_gl_mapping_idx (id, fee_type, gl_code)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.fee_type.clone())
                .bind(idx.gl_code.as_str())
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::FeeTypeGlMapping,
                };
                sqlx::query(
                    r#"
                    INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(audit_link.audit_log_id)
                .bind(audit_link.entity_id)
                .bind(audit_link.entity_type)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.fee_type_gl_mapping_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, FeeTypeGlMappingModel> for FeeTypeGlMappingRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, product::fee_type_gl_mapping::FeeTypeGlMappingModel},
        repository::create_batch::CreateBatch,
    };
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut fee_type_gl_mapping_entities = Vec::new();
        for i in 0..5 {
            let fee_type_gl_mapping = create_test_fee_type_gl_mapping(&format!("GL{i}"));
            fee_type_gl_mapping_entities.push(fee_type_gl_mapping);
        }

        let saved_entities = fee_type_gl_mapping_repo
            .create_batch(fee_type_gl_mapping_entities.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 5);

        for (i, saved_entity) in saved_entities.iter().enumerate() {
            assert_eq!(saved_entity.gl_code.as_str(), format!("GL{i}"));
            assert!(saved_entity.audit_log_id.is_some());
            assert_eq!(saved_entity.audit_log_id.unwrap(), audit_log.id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        let audit_log = create_test_audit_log();
        let saved_entities = fee_type_gl_mapping_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_fee_type_gl_mapping_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test fee_type_gl_mapping entity
        let test_fee_type_gl_mapping = create_test_fee_type_gl_mapping(&random(20));
        let fee_type_gl_mapping_idx = test_fee_type_gl_mapping.to_index();

        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;

        // Insert the fee_type_gl_mapping record
        let audit_log = create_test_audit_log();
        sqlx::query(
            r#"
            INSERT INTO audit_log (id, updated_at, updated_by_person_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_log.id)
        .bind(audit_log.updated_at)
        .bind(audit_log.updated_by_person_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert audit log");
        
        let mut test_fee_type_gl_mapping_for_hashing = test_fee_type_gl_mapping.clone();
        test_fee_type_gl_mapping_for_hashing.hash = 0;
        test_fee_type_gl_mapping_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_fee_type_gl_mapping_for_hashing).unwrap();
        let final_fee_type_gl_mapping = FeeTypeGlMappingModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_fee_type_gl_mapping
        };

        sqlx::query(
            r#"
            INSERT INTO fee_type_gl_mapping
            (id, fee_type, gl_code, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(final_fee_type_gl_mapping.id)
        .bind(final_fee_type_gl_mapping.fee_type.clone())
        .bind(final_fee_type_gl_mapping.gl_code.as_str())
        .bind(final_fee_type_gl_mapping.antecedent_hash)
        .bind(final_fee_type_gl_mapping.antecedent_audit_log_id)
        .bind(final_fee_type_gl_mapping.hash)
        .bind(final_fee_type_gl_mapping.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert fee_type_gl_mapping");

        // Then insert the fee_type_gl_mapping index directly into the database using raw SQL
        sqlx::query("INSERT INTO fee_type_gl_mapping_idx (id, fee_type, gl_code) VALUES ($1, $2, $3)")
            .bind(fee_type_gl_mapping_idx.id)
            .bind(fee_type_gl_mapping_idx.fee_type.clone())
            .bind(fee_type_gl_mapping_idx.gl_code.as_str())
            .execute(&**pool)
            .await
            .expect("Failed to insert fee_type_gl_mapping index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        // Verify the cache was updated via the trigger
        let cache = fee_type_gl_mapping_repo.fee_type_gl_mapping_idx_cache.read().await;
        assert!(
            cache.contains_primary(&fee_type_gl_mapping_idx.id),
            "FeeTypeGlMapping should be in cache after insert"
        );

        let cached_fee_type_gl_mapping = cache.get_by_primary(&fee_type_gl_mapping_idx.id);
        assert!(
            cached_fee_type_gl_mapping.is_some(),
            "FeeTypeGlMapping should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_fee_type_gl_mapping = cached_fee_type_gl_mapping.unwrap();
        assert_eq!(cached_fee_type_gl_mapping.id, fee_type_gl_mapping_idx.id);
        assert_eq!(cached_fee_type_gl_mapping.gl_code, fee_type_gl_mapping_idx.gl_code);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete fee_type_gl_mapping_idx
        sqlx::query("DELETE FROM fee_type_gl_mapping WHERE id = $1")
            .bind(fee_type_gl_mapping_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete fee_type_gl_mapping");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = fee_type_gl_mapping_repo.fee_type_gl_mapping_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&fee_type_gl_mapping_idx.id),
            "FeeTypeGlMapping should be removed from cache after delete"
        );

        Ok(())
    }
}