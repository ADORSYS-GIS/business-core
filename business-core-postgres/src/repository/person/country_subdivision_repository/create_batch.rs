use async_trait::async_trait;
use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &CountrySubdivisionRepositoryImpl,
        items: Vec<CountrySubdivisionModel>,
    ) -> Result<Vec<CountrySubdivisionModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for item in items {
                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO country_subdivision (id, country_id, code, name_l1, name_l2, name_l3)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    "#,
                )
                .bind(item.id)
                .bind(item.country_id)
                .bind(item.code.as_str())
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_ref().map(|s| s.as_str()))
                .bind(item.name_l3.as_ref().map(|s| s.as_str()))
                .execute(&mut **transaction)
                .await?;

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO country_subdivision_idx (id, country_id, code_hash)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.country_id)
                .bind(idx.code_hash)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.country_subdivision_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, CountrySubdivisionModel> for CountrySubdivisionRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<CountrySubdivisionModel>,
        _audit_log_id: Uuid,
    ) -> Result<Vec<CountrySubdivisionModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        let audit_log_id = Uuid::new_v4();
        country_repo.create_batch(vec![country], audit_log_id).await?;

        let mut subdivisions = Vec::new();
        for i in 0..5 {
            let subdivision = create_test_country_subdivision(
                country_id,
                &format!("SD{}", i),
                &format!("Test Subdivision {}", i),
            );
            subdivisions.push(subdivision);
        }

        let audit_log_id = Uuid::new_v4();
        let saved_subdivisions = country_subdivision_repo.create_batch(subdivisions.clone(), audit_log_id).await?;

        assert_eq!(saved_subdivisions.len(), 5);

        for saved_subdivision in &saved_subdivisions {
            assert_eq!(saved_subdivision.country_id, country_id);
            assert!(saved_subdivision.code.as_str().starts_with("SD"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        let audit_log_id = Uuid::new_v4();
        let saved_subdivisions = country_subdivision_repo.create_batch(Vec::new(), audit_log_id).await?;

        assert_eq!(saved_subdivisions.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_country_subdivision_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test country first (required by foreign key)
        let test_country = create_test_country(&random(2), "Test Country");
        let country_id = test_country.id;

        // Create a test country subdivision with a unique code to avoid conflicts
        let unique_code = random(5);
        let test_subdivision = create_test_country_subdivision(country_id, &unique_code, "Test Subdivision");
        let subdivision_idx = test_subdivision.to_index();
    
        // Give listener more time to start and establish connection
        // The listener needs time to connect and execute LISTEN command
        sleep(Duration::from_millis(2000)).await;
    
        // First insert the country record (required by foreign key)
        sqlx::query("INSERT INTO country (id, iso2, name_l1, name_l2, name_l3) VALUES ($1, $2, $3, $4, $5)")
            .bind(test_country.id)
            .bind(test_country.iso2.as_str())
            .bind(test_country.name_l1.as_str())
            .bind(test_country.name_l2.as_ref().map(|s| s.as_str()))
            .bind(test_country.name_l3.as_ref().map(|s| s.as_str()))
            .execute(&**pool)
            .await
            .expect("Failed to insert country");

        // Then insert the country subdivision record
        sqlx::query("INSERT INTO country_subdivision (id, country_id, code, name_l1, name_l2, name_l3) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(test_subdivision.id)
            .bind(test_subdivision.country_id)
            .bind(test_subdivision.code.as_str())
            .bind(test_subdivision.name_l1.as_str())
            .bind(test_subdivision.name_l2.as_ref().map(|s| s.as_str()))
            .bind(test_subdivision.name_l3.as_ref().map(|s| s.as_str()))
            .execute(&**pool)
            .await
            .expect("Failed to insert country subdivision");
    
        // Then insert the country subdivision index directly into the database using raw SQL
        sqlx::query("INSERT INTO country_subdivision_idx (id, country_id, code_hash) VALUES ($1, $2, $3)")
            .bind(subdivision_idx.id)
            .bind(subdivision_idx.country_id)
            .bind(subdivision_idx.code_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert country subdivision index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // Verify the cache was updated via the trigger
        let cache = subdivision_repo.country_subdivision_idx_cache.read().await;
        assert!(
            cache.contains_primary(&subdivision_idx.id),
            "Country subdivision should be in cache after insert"
        );
    
        let cached_subdivision = cache.get_by_primary(&subdivision_idx.id);
        assert!(cached_subdivision.is_some(), "Country subdivision should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_subdivision = cached_subdivision.unwrap();
        assert_eq!(cached_subdivision.id, subdivision_idx.id);
        assert_eq!(cached_subdivision.country_id, subdivision_idx.country_id);
        assert_eq!(cached_subdivision.code_hash, subdivision_idx.code_hash);
        
        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete country_subdivision_idx
        sqlx::query("DELETE FROM country_subdivision WHERE id = $1")
            .bind(subdivision_idx.id)
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
        let cache = subdivision_repo.country_subdivision_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&subdivision_idx.id),
            "Country subdivision should be removed from cache after delete"
        );
        
        Ok(())
    }
}