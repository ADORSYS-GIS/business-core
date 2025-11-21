use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    person::entity_reference::EntityReferenceModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &EntityReferenceRepositoryImpl,
        items: Vec<EntityReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<EntityReferenceModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for EntityReferenceModel")?;
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
                    INSERT INTO entity_reference_audit
                    (id, person_id, entity_role, reference_external_id, reference_details_l1, reference_details_l2, reference_details_l3, related_person_id, start_date, end_date, status, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_id)
                .bind(item.entity_role)
                .bind(item.reference_external_id.as_str())
                .bind(item.reference_details_l1.as_deref())
                .bind(item.reference_details_l2.as_deref())
                .bind(item.reference_details_l3.as_deref())
                .bind(item.related_person_id)
                .bind(item.start_date)
                .bind(item.end_date)
                .bind(item.status)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO entity_reference
                    (id, person_id, entity_role, reference_external_id, reference_details_l1, reference_details_l2, reference_details_l3, related_person_id, start_date, end_date, status, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_id)
                .bind(item.entity_role)
                .bind(item.reference_external_id.as_str())
                .bind(item.reference_details_l1.as_deref())
                .bind(item.reference_details_l2.as_deref())
                .bind(item.reference_details_l3.as_deref())
                .bind(item.related_person_id)
                .bind(item.start_date)
                .bind(item.end_date)
                .bind(item.status)
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
                    INSERT INTO entity_reference_idx (id, person_id, reference_external_id_hash)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.person_id)
                .bind(idx.reference_external_id_hash)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::EntityReference,
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
            let cache = repo.entity_reference_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, EntityReferenceModel> for EntityReferenceRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<EntityReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<EntityReferenceModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::entity_reference_repository::test_utils::create_test_entity_reference;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, person::entity_reference::EntityReferenceModel},
        repository::create_batch::CreateBatch,
    };
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // First create a person (required by foreign key constraint)
        let person = create_test_person("John Doe");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo
            .create_batch(vec![person], Some(audit_log.id))
            .await?;

        let mut entity_references = Vec::new();
        for i in 0..5 {
            let entity_reference = create_test_entity_reference(person_id, &format!("REF-{i}"));
            entity_references.push(entity_reference);
        }

        let saved_entity_references = entity_reference_repo
            .create_batch(entity_references.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entity_references.len(), 5);

        for saved_entity_reference in &saved_entity_references {
            assert_eq!(saved_entity_reference.person_id, person_id);
            assert!(saved_entity_reference
                .reference_external_id
                .as_str()
                .starts_with("REF-"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let audit_log = create_test_audit_log();
        let saved_entity_references = entity_reference_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_entity_references.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_entity_reference_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test person first (required by foreign key)
        let test_person = create_test_person(&random(10));
        let person_id = test_person.id;

        // Create a test entity reference
        let test_entity_reference = create_test_entity_reference(person_id, &format!("REF-{}", random(10)));
        let entity_reference_idx = test_entity_reference.to_index();

        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;

        // First insert the person record (required by foreign key)
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

        let mut test_person_for_hashing = test_person.clone();
        test_person_for_hashing.hash = 0;
        test_person_for_hashing.audit_log_id = Some(audit_log.id);
        let person_computed_hash =
            business_core_db::utils::hash_as_i64(&test_person_for_hashing).unwrap();

        sqlx::query(
            r#"
            INSERT INTO person
            (id, person_type, risk_rating, status, display_name, external_identifier, id_type, id_number, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            "#,
        )
        .bind(test_person.id)
        .bind(test_person.person_type)
        .bind(test_person.risk_rating)
        .bind(test_person.status)
        .bind(test_person.display_name.as_str())
        .bind(test_person.external_identifier.as_deref())
        .bind(test_person.id_type)
        .bind(test_person.id_number.as_str())
        .bind(test_person.entity_reference_count)
        .bind(test_person.organization_person_id)
        .bind(test_person.messaging_info1.as_deref())
        .bind(test_person.messaging_info2.as_deref())
        .bind(test_person.messaging_info3.as_deref())
        .bind(test_person.messaging_info4.as_deref())
        .bind(test_person.messaging_info5.as_deref())
        .bind(test_person.department.as_deref())
        .bind(test_person.location_id)
        .bind(test_person.duplicate_of_person_id)
        .bind(test_person.antecedent_hash)
        .bind(test_person.antecedent_audit_log_id)
        .bind(person_computed_hash)
        .bind(audit_log.id)
        .execute(&**pool)
        .await
        .expect("Failed to insert person");

        // Now insert the entity_reference record
        let mut test_entity_reference_for_hashing = test_entity_reference.clone();
        test_entity_reference_for_hashing.hash = 0;
        test_entity_reference_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_entity_reference_for_hashing).unwrap();
        let final_entity_reference = EntityReferenceModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_entity_reference
        };

        sqlx::query(
            r#"
            INSERT INTO entity_reference
            (id, person_id, entity_role, reference_external_id, reference_details_l1, reference_details_l2, reference_details_l3, related_person_id, start_date, end_date, status, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(final_entity_reference.id)
        .bind(final_entity_reference.person_id)
        .bind(final_entity_reference.entity_role)
        .bind(final_entity_reference.reference_external_id.as_str())
        .bind(final_entity_reference.reference_details_l1.as_deref())
        .bind(final_entity_reference.reference_details_l2.as_deref())
        .bind(final_entity_reference.reference_details_l3.as_deref())
        .bind(final_entity_reference.related_person_id)
        .bind(final_entity_reference.start_date)
        .bind(final_entity_reference.end_date)
        .bind(final_entity_reference.status)
        .bind(final_entity_reference.antecedent_hash)
        .bind(final_entity_reference.antecedent_audit_log_id)
        .bind(final_entity_reference.hash)
        .bind(final_entity_reference.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert entity_reference");

        // Then insert the entity_reference index directly into the database using raw SQL
        sqlx::query("INSERT INTO entity_reference_idx (id, person_id, reference_external_id_hash) VALUES ($1, $2, $3)")
            .bind(entity_reference_idx.id)
            .bind(entity_reference_idx.person_id)
            .bind(entity_reference_idx.reference_external_id_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert entity_reference index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Verify the cache was updated via the trigger
        let cache = entity_reference_repo.entity_reference_idx_cache.read().await;
        assert!(
            cache.contains_primary(&entity_reference_idx.id),
            "EntityReference should be in cache after insert"
        );

        let cached_entity_reference = cache.get_by_primary(&entity_reference_idx.id);
        assert!(
            cached_entity_reference.is_some(),
            "EntityReference should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_entity_reference = cached_entity_reference.unwrap();
        assert_eq!(cached_entity_reference.id, entity_reference_idx.id);
        assert_eq!(cached_entity_reference.person_id, entity_reference_idx.person_id);
        assert_eq!(cached_entity_reference.reference_external_id_hash, entity_reference_idx.reference_external_id_hash);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete entity_reference_idx
        sqlx::query("DELETE FROM entity_reference WHERE id = $1")
            .bind(entity_reference_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete entity_reference");

        sqlx::query("DELETE FROM person WHERE id = $1")
            .bind(person_id)
            .execute(&**pool)
            .await
            .expect("Failed to delete person");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = entity_reference_repo.entity_reference_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&entity_reference_idx.id),
            "EntityReference should be removed from cache after delete"
        );

        Ok(())
    }
}