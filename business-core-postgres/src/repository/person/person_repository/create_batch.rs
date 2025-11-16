use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, EntityType},
    person::person::PersonModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &PersonRepositoryImpl,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for PersonModel")?;
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
                    INSERT INTO person_audit
                    (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.entity_reference_count)
                .bind(item.organization_person_id)
                .bind(item.messaging_info1.as_deref())
                .bind(item.messaging_info2.as_deref())
                .bind(item.messaging_info3.as_deref())
                .bind(item.messaging_info4.as_deref())
                .bind(item.messaging_info5.as_deref())
                .bind(item.department.as_deref())
                .bind(item.location_id)
                .bind(item.duplicate_of_person_id)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO person
                    (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.entity_reference_count)
                .bind(item.organization_person_id)
                .bind(item.messaging_info1.as_deref())
                .bind(item.messaging_info2.as_deref())
                .bind(item.messaging_info3.as_deref())
                .bind(item.messaging_info4.as_deref())
                .bind(item.messaging_info5.as_deref())
                .bind(item.department.as_deref())
                .bind(item.location_id)
                .bind(item.duplicate_of_person_id)
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
                    INSERT INTO person_idx (id, external_identifier_hash, organization_person_id, duplicate_of_person_id)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.external_identifier_hash)
                .bind(idx.organization_person_id)
                .bind(idx.duplicate_of_person_id)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: EntityType::Person,
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
            let cache = repo.person_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, PersonModel> for PersonRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, person::person::{PersonModel, PersonType}},
        repository::create_batch::CreateBatch,
    };
    use crate::repository::person::person_repository::test_utils::{
        create_test_person, create_test_person_with_external_id,
    };
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut persons = Vec::new();
        for i in 0..5 {
            let person = create_test_person(
                &format!("Person {}", i),
                PersonType::Natural,
            );
            persons.push(person);
        }

        let saved_persons = person_repo
            .create_batch(persons.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_persons.len(), 5);

        for (i, saved_person) in saved_persons.iter().enumerate() {
            assert_eq!(saved_person.display_name.as_str(), format!("Person {}", i));
            assert_eq!(saved_person.person_type, PersonType::Natural);
            assert!(saved_person.audit_log_id.is_some());
            assert_eq!(saved_person.audit_log_id.unwrap(), audit_log.id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        let saved_persons = person_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_persons.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_with_external_identifier() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut persons = Vec::new();
        for i in 0..3 {
            let person = create_test_person_with_external_id(
                &format!("Employee {}", i),
                PersonType::Natural,
                &format!("EMP{:03}", i),
            );
            persons.push(person);
        }

        let saved_persons = person_repo
            .create_batch(persons.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_persons.len(), 3);

        for (i, saved_person) in saved_persons.iter().enumerate() {
            assert!(saved_person.external_identifier.is_some());
            assert_eq!(
                saved_person.external_identifier.as_ref().unwrap().as_str(),
                format!("EMP{:03}", i)
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_person_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test person
        let test_person = create_test_person(&random(20), PersonType::Natural);
        let person_idx = test_person.to_index();

        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;

        // Insert the person record
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
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_person_for_hashing).unwrap();
        let final_person = PersonModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_person
        };

        sqlx::query(
            r#"
            INSERT INTO person
            (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(final_person.id)
        .bind(final_person.person_type)
        .bind(final_person.display_name.as_str())
        .bind(final_person.external_identifier.as_deref())
        .bind(final_person.entity_reference_count)
        .bind(final_person.organization_person_id)
        .bind(final_person.messaging_info1.as_deref())
        .bind(final_person.messaging_info2.as_deref())
        .bind(final_person.messaging_info3.as_deref())
        .bind(final_person.messaging_info4.as_deref())
        .bind(final_person.messaging_info5.as_deref())
        .bind(final_person.department.as_deref())
        .bind(final_person.location_id)
        .bind(final_person.duplicate_of_person_id)
        .bind(final_person.antecedent_hash)
        .bind(final_person.antecedent_audit_log_id)
        .bind(final_person.hash)
        .bind(final_person.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert person");

        // Then insert the person index directly into the database using raw SQL
        sqlx::query("INSERT INTO person_idx (id, external_identifier_hash, organization_person_id, duplicate_of_person_id) VALUES ($1, $2, $3, $4)")
            .bind(person_idx.id)
            .bind(person_idx.external_identifier_hash)
            .bind(person_idx.organization_person_id)
            .bind(person_idx.duplicate_of_person_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert person index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let person_repo = &ctx.person_repos().person_repository;

        // Verify the cache was updated via the trigger
        let cache = person_repo.person_idx_cache.read().await;
        assert!(
            cache.contains_primary(&person_idx.id),
            "Person should be in cache after insert"
        );

        let cached_person = cache.get_by_primary(&person_idx.id);
        assert!(
            cached_person.is_some(),
            "Person should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_person = cached_person.unwrap();
        assert_eq!(cached_person.id, person_idx.id);
        assert_eq!(cached_person.external_identifier_hash, person_idx.external_identifier_hash);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete person_idx
        sqlx::query("DELETE FROM person WHERE id = $1")
            .bind(person_idx.id)
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
        let cache = person_repo.person_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&person_idx.id),
            "Person should be removed from cache after delete"
        );

        Ok(())
    }
}