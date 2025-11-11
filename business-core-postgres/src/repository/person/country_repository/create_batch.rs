use async_trait::async_trait;
use business_core_db::models::person::country::CountryModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &CountryRepositoryImpl,
        items: Vec<CountryModel>,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
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
                    INSERT INTO country (id, iso2, name_l1, name_l2, name_l3)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(item.id)
                .bind(item.iso2.as_str())
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_ref().map(|s| s.as_str()))
                .bind(item.name_l3.as_ref().map(|s| s.as_str()))
                .execute(&mut **transaction)
                .await?;

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO country_idx (id, iso2_hash)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.iso2_hash)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let mut cache = repo.country_idx_cache.write();
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, CountryModel> for CountryRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<CountryModel>,
        _audit_log_id: Uuid,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("C{}", i),
                &format!("Test Country {}", i),
            );
            countries.push(country);
        }

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(countries.clone(), audit_log_id).await?;

        assert_eq!(saved_countries.len(), 5);

        for saved_country in &saved_countries {
            assert_eq!(saved_country.iso2.as_str().chars().next().unwrap(), 'C');
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(Vec::new(), audit_log_id).await?;

        assert_eq!(saved_countries.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_country_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test country with a unique ISO2 code to avoid conflicts
        let unique_iso2 = {
            let uuid = uuid::Uuid::new_v4();
            let uuid_bytes = uuid.as_bytes();
            let char1 = (b'A' + (uuid_bytes[0] % 26)) as char;
            let char2 = (b'A' + (uuid_bytes[1] % 26)) as char;
            format!("{}{}", char1, char2)
        };
        let test_country = create_test_country(&unique_iso2[..2], "Test Country");
        let country_idx = test_country.to_index();
    
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
    
        // Then insert the country index directly into the database using raw SQL
        sqlx::query("INSERT INTO country_idx (id, iso2_hash) VALUES ($1, $2)")
            .bind(country_idx.id)
            .bind(country_idx.iso2_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert country index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let country_repo = &ctx.person_repos().country_repository;

        // Verify the cache was updated via the trigger
        let cache = country_repo.country_idx_cache.read();
        assert!(
            cache.contains_primary(&country_idx.id),
            "Country should be in cache after insert"
        );
    
        let cached_country = cache.get_by_primary(&country_idx.id);
        assert!(cached_country.is_some(), "Country should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_country = cached_country.unwrap();
        assert_eq!(cached_country.id, country_idx.id);
        assert_eq!(cached_country.iso2_hash, country_idx.iso2_hash);
        
        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete country_idx
        sqlx::query("DELETE FROM country WHERE id = $1")
            .bind(country_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete country");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = country_repo.country_idx_cache.read();
        assert!(
            !cache.contains_primary(&country_idx.id),
            "Country should be removed from cache after delete"
        );
        
        Ok(())
    }
}