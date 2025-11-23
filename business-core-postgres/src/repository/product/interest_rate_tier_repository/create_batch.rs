use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditEntityType, AuditLinkModel},
    index_aware::IndexAware,
    product::interest_rate_tier::InterestRateTierModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &InterestRateTierRepositoryImpl,
        items: Vec<InterestRateTierModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for InterestRateTierModel")?;
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
                entity_for_hashing.hash = 0; // Must be 0 before hashing
                entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing

                // 2. Compute hash
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                // 3. Update original entity with computed hash and new audit_log_id
                item.hash = computed_hash;
                item.audit_log_id = Some(audit_log_id);

                // Execute audit insert
                sqlx::query(
                    r#"
                    INSERT INTO interest_rate_tier_audit
                    (id, name, minimum_balance, maximum_balance, interest_rate, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.interest_rate)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO interest_rate_tier
                    (id, name, minimum_balance, maximum_balance, interest_rate, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.interest_rate)
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
                    INSERT INTO interest_rate_tier_idx (id, name, minimum_balance, maximum_balance, interest_rate)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.name)
                .bind(idx.minimum_balance)
                .bind(idx.maximum_balance)
                .bind(idx.interest_rate)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::InterestRateTier,
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
            let cache = repo.interest_rate_tier_idx_cache.write().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, InterestRateTierModel> for InterestRateTierRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<InterestRateTierModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::{setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, product::interest_rate_tier::InterestRateTierModel},
        repository::create_batch::CreateBatch,
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut interest_rate_tiers = Vec::new();
        for _ in 0..5 {
            let interest_rate_tier = create_test_interest_rate_tier();
            interest_rate_tiers.push(interest_rate_tier);
        }

        let saved_entities = interest_rate_tier_repo
            .create_batch(interest_rate_tiers.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 5);

        for saved_entity in saved_entities.iter() {
            assert!(saved_entity.audit_log_id.is_some());
            assert_eq!(saved_entity.audit_log_id.unwrap(), audit_log.id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        let audit_log = create_test_audit_log();
        let saved_entities = interest_rate_tier_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entities.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_interest_rate_tier_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test interest_rate_tier entity
        let test_interest_rate_tier = create_test_interest_rate_tier();
        let interest_rate_tier_idx = test_interest_rate_tier.to_index();

        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;

        // Insert the interest_rate_tier record
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

        let mut test_interest_rate_tier_for_hashing = test_interest_rate_tier.clone();
        test_interest_rate_tier_for_hashing.hash = 0;
        test_interest_rate_tier_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_interest_rate_tier_for_hashing).unwrap();
        let final_interest_rate_tier = InterestRateTierModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_interest_rate_tier
        };

        sqlx::query(
            r#"
            INSERT INTO interest_rate_tier
            (id, name, minimum_balance, maximum_balance, interest_rate, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(final_interest_rate_tier.id)
        .bind(final_interest_rate_tier.name)
        .bind(final_interest_rate_tier.minimum_balance)
        .bind(final_interest_rate_tier.maximum_balance)
        .bind(final_interest_rate_tier.interest_rate)
        .bind(final_interest_rate_tier.antecedent_hash)
        .bind(final_interest_rate_tier.antecedent_audit_log_id)
        .bind(final_interest_rate_tier.hash)
        .bind(final_interest_rate_tier.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert interest_rate_tier");

        // Then insert the interest_rate_tier index directly into the database using raw SQL
        sqlx::query("INSERT INTO interest_rate_tier_idx (id, name, minimum_balance, maximum_balance, interest_rate) VALUES ($1, $2, $3, $4, $5)")
            .bind(interest_rate_tier_idx.id)
            .bind(interest_rate_tier_idx.name)
            .bind(interest_rate_tier_idx.minimum_balance)
            .bind(interest_rate_tier_idx.maximum_balance)
            .bind(interest_rate_tier_idx.interest_rate)
            .execute(&**pool)
            .await
            .expect("Failed to insert interest_rate_tier index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        // Verify the cache was updated via the trigger
        let cache = interest_rate_tier_repo.interest_rate_tier_idx_cache.read().await;
        assert!(
            cache.contains_primary(&interest_rate_tier_idx.id),
            "InterestRateTier should be in cache after insert"
        );

        let cached_interest_rate_tier = cache.get_by_primary(&interest_rate_tier_idx.id);
        assert!(
            cached_interest_rate_tier.is_some(),
            "InterestRateTier should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_interest_rate_tier = cached_interest_rate_tier.unwrap();
        assert_eq!(cached_interest_rate_tier.id, interest_rate_tier_idx.id);
        assert_eq!(cached_interest_rate_tier.name, interest_rate_tier_idx.name);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete interest_rate_tier_idx
        sqlx::query("DELETE FROM interest_rate_tier WHERE id = $1")
            .bind(interest_rate_tier_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete interest_rate_tier");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = interest_rate_tier_repo.interest_rate_tier_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&interest_rate_tier_idx.id),
            "InterestRateTier should be removed from cache after delete"
        );

        Ok(())
    }
}