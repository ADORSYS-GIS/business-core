use async_trait::async_trait;
use business_core_db::models::person::location::LocationModel;
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
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
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
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::{repository::create_batch::CreateBatch};
    use super::super::test_utils::{create_test_audit_log, create_test_country, create_test_country_subdivision, create_test_locality, create_test_location};

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
        country_repo.create_batch(vec![country], audit_log.id).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "CA", "California");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], audit_log.id).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "SF", "San Francisco");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], audit_log.id).await?;

        let mut locations = Vec::new();
        for i in 0..5 {
            let location = create_test_location(
                locality_id,
                &format!("{} Main St", i),
            );
            locations.push(location);
        }

        let saved_locations = location_repo.create_batch(locations.clone(), audit_log.id).await?;

        assert_eq!(saved_locations.len(), 5);

        for saved_location in &saved_locations {
            assert_eq!(saved_location.locality_id, locality_id);
            assert!(saved_location.street_line1.as_str().ends_with(" Main St"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let location_repo = &ctx.person_repos().location_repository;

        let audit_log = create_test_audit_log();
        let saved_locations = location_repo.create_batch(Vec::new(), audit_log.id).await?;

        assert_eq!(saved_locations.len(), 0);

        Ok(())
    }
}