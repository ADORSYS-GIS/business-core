use async_trait::async_trait;
use business_core_db::models::calendar::weekend_days::WeekendDaysModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &WeekendDaysRepositoryImpl,
        items: Vec<WeekendDaysModel>,
    ) -> Result<Vec<WeekendDaysModel>, Box<dyn Error + Send + Sync>> {
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
                    INSERT INTO calendar_weekend_days (
                        id, country_id, country_subdivision_id,
                        weekend_day_01, weekend_day_02, weekend_day_03, weekend_day_04,
                        weekend_day_05, weekend_day_06, weekend_day_07,
                        effective_date, expiry_date
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    "#,
                )
                .bind(item.id)
                .bind(item.country_id)
                .bind(item.country_subdivision_id)
                .bind(item.weekend_day_01)
                .bind(item.weekend_day_02)
                .bind(item.weekend_day_03)
                .bind(item.weekend_day_04)
                .bind(item.weekend_day_05)
                .bind(item.weekend_day_06)
                .bind(item.weekend_day_07)
                .bind(item.effective_date)
                .bind(item.expiry_date)
                .execute(&mut **transaction)
                .await?;

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO calendar_weekend_days_idx (id, country_id, country_subdivision_id)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.country_id)
                .bind(idx.country_subdivision_id)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.weekend_days_idx_cache.read().await;
            let main_cache = repo.weekend_days_cache.read().await;
            
            for (idx, item) in indices.iter().zip(saved_items.iter()) {
                idx_cache.add(idx.clone());
                main_cache.insert(item.clone());
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, WeekendDaysModel> for WeekendDaysRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<WeekendDaysModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<WeekendDaysModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let mut items = Vec::new();
        for _ in 0..3 {
            let item = create_test_weekend_days(None, None);
            items.push(item);
        }

        let saved_items = weekend_days_repo.create_batch(items.clone(), None).await?;

        assert_eq!(saved_items.len(), 3);

        // Verify items are in main cache
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        for item in &saved_items {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let saved_items = weekend_days_repo.create_batch(Vec::new(), None).await?;

        assert_eq!(saved_items.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_weekend_days_insert_triggers_index_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test weekend_days entity
        let test_item = create_test_weekend_days(None, None);
        let item_idx = test_item.to_index();
    
        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // First insert the main record
        sqlx::query(
            r#"
            INSERT INTO calendar_weekend_days (
                id, country_id, country_subdivision_id,
                weekend_day_01, weekend_day_02, weekend_day_03, weekend_day_04,
                weekend_day_05, weekend_day_06, weekend_day_07,
                effective_date, expiry_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(test_item.id)
        .bind(test_item.country_id)
        .bind(test_item.country_subdivision_id)
        .bind(test_item.weekend_day_01)
        .bind(test_item.weekend_day_02)
        .bind(test_item.weekend_day_03)
        .bind(test_item.weekend_day_04)
        .bind(test_item.weekend_day_05)
        .bind(test_item.weekend_day_06)
        .bind(test_item.weekend_day_07)
        .bind(test_item.effective_date)
        .bind(test_item.expiry_date)
        .execute(&**pool)
        .await
        .expect("Failed to insert weekend_days");
    
        // Then insert the index directly into the database using raw SQL
        sqlx::query("INSERT INTO calendar_weekend_days_idx (id, country_id, country_subdivision_id) VALUES ($1, $2, $3)")
            .bind(item_idx.id)
            .bind(item_idx.country_id)
            .bind(item_idx.country_subdivision_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert weekend_days index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        // Verify the index cache was updated via the trigger
        let cache = weekend_days_repo.weekend_days_idx_cache.read().await;
        assert!(
            cache.contains_primary(&item_idx.id),
            "WeekendDays index should be in cache after insert"
        );
    
        let cached_idx = cache.get_by_primary(&item_idx.id);
        assert!(cached_idx.is_some(), "WeekendDays index should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_idx = cached_idx.unwrap();
        assert_eq!(cached_idx.id, item_idx.id);
        assert_eq!(cached_idx.country_id, item_idx.country_id);
        assert_eq!(cached_idx.country_subdivision_id, item_idx.country_subdivision_id);
        
        // Drop the read lock before proceeding
        drop(cache);

        // Delete the record from the database
        sqlx::query("DELETE FROM calendar_weekend_days WHERE id = $1")
            .bind(item_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete weekend_days");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = weekend_days_repo.weekend_days_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&item_idx.id),
            "WeekendDays index should be removed from cache after delete"
        );
        
        Ok(())
    }

    #[tokio::test]
    async fn test_weekend_days_insert_triggers_main_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test weekend_days entity
        let test_item = create_test_weekend_days(None, None);
    
        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // Insert the main record directly into the database using raw SQL
        sqlx::query(
            r#"
            INSERT INTO calendar_weekend_days (
                id, country_id, country_subdivision_id,
                weekend_day_01, weekend_day_02, weekend_day_03, weekend_day_04,
                weekend_day_05, weekend_day_06, weekend_day_07,
                effective_date, expiry_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(test_item.id)
        .bind(test_item.country_id)
        .bind(test_item.country_subdivision_id)
        .bind(test_item.weekend_day_01)
        .bind(test_item.weekend_day_02)
        .bind(test_item.weekend_day_03)
        .bind(test_item.weekend_day_04)
        .bind(test_item.weekend_day_05)
        .bind(test_item.weekend_day_06)
        .bind(test_item.weekend_day_07)
        .bind(test_item.effective_date)
        .bind(test_item.expiry_date)
        .execute(&**pool)
        .await
        .expect("Failed to insert weekend_days");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        // Verify the main cache was updated via the trigger
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        assert!(
            main_cache.contains(&test_item.id),
            "WeekendDays should be in main cache after insert"
        );
    
        let cached_item = main_cache.get(&test_item.id);
        assert!(cached_item.is_some(), "WeekendDays should be retrievable from main cache");
        
        // Verify the cached data matches
        let cached_item = cached_item.unwrap();
        assert_eq!(cached_item.id, test_item.id);
        assert_eq!(cached_item.country_id, test_item.country_id);
        assert_eq!(cached_item.country_subdivision_id, test_item.country_subdivision_id);
        assert_eq!(cached_item.weekend_day_01, test_item.weekend_day_01);
        assert_eq!(cached_item.weekend_day_02, test_item.weekend_day_02);
        
        // Drop the read lock before proceeding
        drop(main_cache);

        // Delete the record from the database
        sqlx::query("DELETE FROM calendar_weekend_days WHERE id = $1")
            .bind(test_item.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete weekend_days");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        assert!(
            !main_cache.contains(&test_item.id),
            "WeekendDays should be removed from main cache after delete"
        );
        
        Ok(())
    }
}