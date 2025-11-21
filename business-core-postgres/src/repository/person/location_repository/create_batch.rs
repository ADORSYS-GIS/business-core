use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    person::location::LocationModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &LocationRepositoryImpl,
        items: Vec<LocationModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for LocationModel")?;
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
                    INSERT INTO location_audit
                    (id, street_line1, street_line2, street_line3, street_line4, locality_id, postal_code, latitude, longitude, accuracy_meters, location_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item.id)
                .bind(item.street_line1.as_str())
                .bind(item.street_line2.as_deref())
                .bind(item.street_line3.as_deref())
                .bind(item.street_line4.as_deref())
                .bind(item.locality_id)
                .bind(item.postal_code.as_deref())
                .bind(item.latitude)
                .bind(item.longitude)
                .bind(item.accuracy_meters)
                .bind(item.location_type)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO location
                    (id, street_line1, street_line2, street_line3, street_line4, locality_id, postal_code, latitude, longitude, accuracy_meters, location_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item.id)
                .bind(item.street_line1.as_str())
                .bind(item.street_line2.as_deref())
                .bind(item.street_line3.as_deref())
                .bind(item.street_line4.as_deref())
                .bind(item.locality_id)
                .bind(item.postal_code.as_deref())
                .bind(item.latitude)
                .bind(item.longitude)
                .bind(item.accuracy_meters)
                .bind(item.location_type)
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
                    INSERT INTO location_idx (id, locality_id)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.locality_id)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::Location,
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
            let cache = repo.location_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, LocationModel> for LocationRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<LocationModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::{
            create_test_audit_log, create_test_country, create_test_country_subdivision,
            create_test_locality, create_test_location,
        };
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::{
        models::{index_aware::IndexAware, person::location::LocationModel},
        repository::create_batch::CreateBatch,
    };
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo
            .create_batch(vec![country], Some(audit_log.id))
            .await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "CA", "California");
        let subdivision_id = subdivision.id;
        country_subdivision_repo
            .create_batch(vec![subdivision], Some(audit_log.id))
            .await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "SF", "San Francisco");
        let locality_id = locality.id;
        locality_repo
            .create_batch(vec![locality], Some(audit_log.id))
            .await?;

        let mut locations = Vec::new();
        for i in 0..5 {
            let location = create_test_location(locality_id, &format!("{i} Main St"));
            locations.push(location);
        }

        let saved_locations = location_repo
            .create_batch(locations.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_locations.len(), 5);

        for saved_location in &saved_locations {
            assert_eq!(saved_location.locality_id, locality_id);
            assert!(saved_location
                .street_line1
                .as_str()
                .ends_with(" Main St"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let location_repo = &ctx.person_repos().location_repository;

        let audit_log = create_test_audit_log();
        let saved_locations = location_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_locations.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_location_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test country first (required by foreign key)
        let test_country = create_test_country(&random(2), "Test Country");
        let country_id = test_country.id;

        // Create a test country subdivision (required by foreign key)
        let test_subdivision =
            create_test_country_subdivision(country_id, &random(5), "Test Subdivision");
        let subdivision_id = test_subdivision.id;

        // Create a test locality (required by foreign key)
        let test_locality = create_test_locality(subdivision_id, &random(5), "Test Locality");
        let locality_id = test_locality.id;

        // Create a test location
        let test_location = create_test_location(locality_id, "123 Main St");
        let location_idx = test_location.to_index();

        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;

        // First insert the country record (required by foreign key)
        sqlx::query("INSERT INTO country (id, iso2, name) VALUES ($1, $2, $3)")
            .bind(test_country.id)
            .bind(test_country.iso2.as_str())
            .bind(test_country.name)
            .execute(&**pool)
            .await
            .expect("Failed to insert country");

        // Then insert the country subdivision record
        sqlx::query("INSERT INTO country_subdivision (id, country_id, code, name) VALUES ($1, $2, $3, $4)")
            .bind(test_subdivision.id)
            .bind(test_subdivision.country_id)
            .bind(test_subdivision.code.as_str())
            .bind(test_subdivision.name)
            .execute(&**pool)
            .await
            .expect("Failed to insert country subdivision");

        // Then insert the locality record
        sqlx::query("INSERT INTO locality (id, country_subdivision_id, code, name_l1, name_l2, name_l3) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(test_locality.id)
            .bind(test_locality.country_subdivision_id)
            .bind(test_locality.code.as_str())
            .bind(test_locality.name_l1.as_str())
            .bind(test_locality.name_l2.as_ref().map(|s| s.as_str()))
            .bind(test_locality.name_l3.as_ref().map(|s| s.as_str()))
            .execute(&**pool)
            .await
            .expect("Failed to insert locality");

        // Then insert the location record
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
        let mut test_location_for_hashing = test_location.clone();
        test_location_for_hashing.hash = 0;
        test_location_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash =
            business_core_db::utils::hash_as_i64(&test_location_for_hashing).unwrap();
        let final_location = LocationModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_location
        };

        sqlx::query(
            r#"
            INSERT INTO location
            (id, street_line1, street_line2, street_line3, street_line4, locality_id, postal_code, latitude, longitude, accuracy_meters, location_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(final_location.id)
        .bind(final_location.street_line1.as_str())
        .bind(final_location.street_line2.as_deref())
        .bind(final_location.street_line3.as_deref())
        .bind(final_location.street_line4.as_deref())
        .bind(final_location.locality_id)
        .bind(final_location.postal_code.as_deref())
        .bind(final_location.latitude)
        .bind(final_location.longitude)
        .bind(final_location.accuracy_meters)
        .bind(final_location.location_type)
        .bind(final_location.antecedent_hash)
        .bind(final_location.antecedent_audit_log_id)
        .bind(final_location.hash)
        .bind(final_location.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert location");

        // Then insert the location index directly into the database using raw SQL
        sqlx::query("INSERT INTO location_idx (id, locality_id) VALUES ($1, $2)")
            .bind(location_idx.id)
            .bind(location_idx.locality_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert location index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let location_repo = &ctx.person_repos().location_repository;

        // Verify the cache was updated via the trigger
        let cache = location_repo.location_idx_cache.read().await;
        assert!(
            cache.contains_primary(&location_idx.id),
            "Location should be in cache after insert"
        );

        let cached_location = cache.get_by_primary(&location_idx.id);
        assert!(
            cached_location.is_some(),
            "Location should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_location = cached_location.unwrap();
        assert_eq!(cached_location.id, location_idx.id);
        assert_eq!(cached_location.locality_id, location_idx.locality_id);

        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete location_idx
        sqlx::query("DELETE FROM location WHERE id = $1")
            .bind(location_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete location");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        sqlx::query("DELETE FROM locality WHERE id = $1")
            .bind(locality_id)
            .execute(&**pool)
            .await
            .expect("Failed to delete locality");

        sqlx::query("DELETE FROM country_subdivision WHERE id = $1")
            .bind(subdivision_id)
            .execute(&**pool)
            .await
            .expect("Failed to delete country subdivision");

        sqlx::query("DELETE FROM country WHERE id = $1")
            .bind(country_id)
            .execute(&**pool)
            .await
            .expect("Failed to delete country");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = location_repo.location_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&location_idx.id),
            "Location should be removed from cache after delete"
        );

        Ok(())
    }
}