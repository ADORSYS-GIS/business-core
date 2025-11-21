use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    description::named::NamedModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::NamedRepositoryImpl;

impl NamedRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &NamedRepositoryImpl,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for NamedModel")?;
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
                    INSERT INTO named_audit
                    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                    "#,
                )
                .bind(item.id)
                .bind(item.entity_type)
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_deref())
                .bind(item.name_l3.as_deref())
                .bind(item.name_l4.as_deref())
                .bind(item.description_l1.as_deref())
                .bind(item.description_l2.as_deref())
                .bind(item.description_l3.as_deref())
                .bind(item.description_l4.as_deref())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO named
                    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                    "#,
                )
                .bind(item.id)
                .bind(item.entity_type)
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_deref())
                .bind(item.name_l3.as_deref())
                .bind(item.name_l4.as_deref())
                .bind(item.description_l1.as_deref())
                .bind(item.description_l2.as_deref())
                .bind(item.description_l3.as_deref())
                .bind(item.description_l4.as_deref())
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
                    INSERT INTO named_idx (id, entity_type)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.entity_type)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::Named,
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
            let cache = repo.named_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, NamedModel> for NamedRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, description::named::NamedModel},
        repository::create_batch::CreateBatch,
    };
    use crate::repository::description::named_repository::test_utils::create_test_named;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut named_entities = Vec::new();
        for i in 0..5 {
            let named = create_test_named(&format!("Entity {i}"));
            named_entities.push(named);
        }

        let saved_entities = named_repo
            .create_batch(named_entities.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 5);

        for (i, saved_entity) in saved_entities.iter().enumerate() {
            assert_eq!(saved_entity.name_l1.as_str(), format!("Entity {i}"));
            assert!(saved_entity.audit_log_id.is_some());
            assert_eq!(saved_entity.audit_log_id.unwrap(), audit_log.id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        let saved_entities = named_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_named_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test named entity
        let test_named = create_test_named(&random(20));
        let named_idx = test_named.to_index();

        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;

        // Insert the named record
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
        
        let mut test_named_for_hashing = test_named.clone();
        test_named_for_hashing.hash = 0;
        test_named_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_named_for_hashing).unwrap();
        let final_named = NamedModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_named
        };

        sqlx::query(
            r#"
            INSERT INTO named
            (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(final_named.id)
        .bind(final_named.entity_type)
        .bind(final_named.name_l1.as_str())
        .bind(final_named.name_l2.as_deref())
        .bind(final_named.name_l3.as_deref())
        .bind(final_named.name_l4.as_deref())
        .bind(final_named.description_l1.as_deref())
        .bind(final_named.description_l2.as_deref())
        .bind(final_named.description_l3.as_deref())
        .bind(final_named.description_l4.as_deref())
        .bind(final_named.antecedent_hash)
        .bind(final_named.antecedent_audit_log_id)
        .bind(final_named.hash)
        .bind(final_named.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert named");

        // Then insert the named index directly into the database using raw SQL
        sqlx::query("INSERT INTO named_idx (id, entity_type) VALUES ($1, $2)")
            .bind(named_idx.id)
            .bind(named_idx.entity_type)
            .execute(&**pool)
            .await
            .expect("Failed to insert named index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let named_repo = &ctx.description_repos().named_repository;

        // Verify the cache was updated via the trigger
        let cache = named_repo.named_idx_cache.read().await;
        assert!(
            cache.contains_primary(&named_idx.id),
            "Named should be in cache after insert"
        );

        let cached_named = cache.get_by_primary(&named_idx.id);
        assert!(
            cached_named.is_some(),
            "Named should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_named = cached_named.unwrap();
        assert_eq!(cached_named.id, named_idx.id);
        assert_eq!(cached_named.entity_type, named_idx.entity_type);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete named_idx
        sqlx::query("DELETE FROM named WHERE id = $1")
            .bind(named_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete named");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = named_repo.named_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&named_idx.id),
            "Named should be removed from cache after delete"
        );

        Ok(())
    }
}