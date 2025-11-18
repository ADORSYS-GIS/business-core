use business_core_db::models::calendar::business_day::BusinessDayModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::create_batch::CreateBatch;
use crate::utils::TryFromRow;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl CreateBatch for BusinessDayRepositoryImpl {
    type Model = BusinessDayModel;

    async fn create_batch(
        &self,
        items: Vec<Self::Model>,
        _audit_info: Option<Uuid>,
    ) -> Result<Vec<Self::Model>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &BusinessDayRepositoryImpl,
        items: Vec<BusinessDayModel>,
    ) -> Result<Vec<BusinessDayModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute main insert
            sqlx::query(
                r#"
                INSERT INTO calendar_business_day (
                    id, country_id, country_subdivision_id, date, weekday,
                    is_business_day, is_weekend, weekend_day_01, is_holiday,
                    holiday_name, day_scope
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(item.id)
            .bind(item.country_id)
            .bind(item.country_subdivision_id)
            .bind(item.date)
            .bind(item.weekday)
            .bind(item.is_business_day)
            .bind(item.is_weekend)
            .bind(item.weekend_day_01)
            .bind(item.is_holiday)
            .bind(item.holiday_name.as_ref().map(|s| s.as_str()))
            .bind(item.day_scope)
            .execute(&mut **transaction)
            .await?;
            
            // Insert into index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                INSERT INTO calendar_business_day_idx (id, country_id, country_subdivision_id, date_hash)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(idx.id)
            .bind(idx.country_id)
            .bind(idx.country_subdivision_id)
            .bind(idx.date_hash)
            .execute(&mut **transaction)
            .await?;
            
            indices.push(idx);
            saved_items.push(item);
        }
        
        // Release transaction lock before updating caches
        drop(tx);
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.business_day_idx_cache.read().await;
            let main_cache = repo.business_day_cache.read().await;
            
            for (idx, item) in indices.iter().zip(saved_items.iter()) {
                idx_cache.add(idx.clone());
                main_cache.insert(item.clone());
            }
        }

        Ok(saved_items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_business_day;

    #[tokio::test]
    async fn test_create_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        let items = vec![
            create_test_business_day(Some(Uuid::new_v4()), None),
            create_test_business_day(None, Some(Uuid::new_v4())),
        ];
        let saved = business_day_repo.create_batch(items, None).await?;

        // Verify entities are in main cache
        let main_cache = business_day_repo.business_day_cache.read().await;
        for item in &saved {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_business_day_insert_triggers_index_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::test_helper::setup_test_context_and_listen;
        use business_core_db::models::index_aware::IndexAware;
        use tokio::time::{sleep, Duration};
        
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test entity
        let test_item = create_test_business_day(Some(Uuid::new_v4()), None);
        let item_idx = test_item.to_index();

        // Give listener time to start
        sleep(Duration::from_millis(2000)).await;

        // First insert the main record
        sqlx::query(
            r#"
            INSERT INTO calendar_business_day (
                id, country_id, country_subdivision_id, date, weekday,
                is_business_day, is_weekend, weekend_day_01, is_holiday,
                holiday_name, day_scope
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(test_item.id)
        .bind(test_item.country_id)
        .bind(test_item.country_subdivision_id)
        .bind(test_item.date)
        .bind(test_item.weekday)
        .bind(test_item.is_business_day)
        .bind(test_item.is_weekend)
        .bind(test_item.weekend_day_01)
        .bind(test_item.is_holiday)
        .bind(test_item.holiday_name.as_ref().map(|s| s.as_str()))
        .bind(test_item.day_scope)
        .execute(&**pool)
        .await
        .expect("Failed to insert business_day");

        // Then insert the index directly into the database using raw SQL
        sqlx::query("INSERT INTO calendar_business_day_idx (id, country_id, country_subdivision_id, date_hash) VALUES ($1, $2, $3, $4)")
            .bind(item_idx.id)
            .bind(item_idx.country_id)
            .bind(item_idx.country_subdivision_id)
            .bind(item_idx.date_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert business_day index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        // Verify the index cache was updated via the trigger
        let cache = business_day_repo.business_day_idx_cache.read().await;
        assert!(
            cache.contains_primary(&item_idx.id),
            "BusinessDay index should be in cache after insert"
        );

        let cached_idx = cache.get_by_primary(&item_idx.id);
        assert!(cached_idx.is_some(), "BusinessDay index should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_idx = cached_idx.unwrap();
        assert_eq!(cached_idx.id, item_idx.id);
        assert_eq!(cached_idx.country_id, item_idx.country_id);
        assert_eq!(cached_idx.country_subdivision_id, item_idx.country_subdivision_id);
        assert_eq!(cached_idx.date_hash, item_idx.date_hash);
        
        // Drop the read lock before proceeding
        drop(cache);

        // Delete the record from the database
        sqlx::query("DELETE FROM calendar_business_day WHERE id = $1")
            .bind(item_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete business_day");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = business_day_repo.business_day_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&item_idx.id),
            "BusinessDay index should be removed from cache after delete"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_business_day_insert_triggers_main_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::test_helper::setup_test_context_and_listen;
        use business_core_db::models::index_aware::IndexAware;
        use tokio::time::{sleep, Duration};
        
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test entity
        let test_item = create_test_business_day(Some(Uuid::new_v4()), None);
        let item_idx = test_item.to_index();

        // Give listener time to start
        sleep(Duration::from_millis(2000)).await;

        // Insert the entity record directly into database (triggers main cache notification)
        sqlx::query(
            r#"
            INSERT INTO calendar_business_day (
                id, country_id, country_subdivision_id, date, weekday,
                is_business_day, is_weekend, weekend_day_01, is_holiday,
                holiday_name, day_scope
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(test_item.id)
        .bind(test_item.country_id)
        .bind(test_item.country_subdivision_id)
        .bind(test_item.date)
        .bind(test_item.weekday)
        .bind(test_item.is_business_day)
        .bind(test_item.is_weekend)
        .bind(test_item.weekend_day_01)
        .bind(test_item.is_holiday)
        .bind(test_item.holiday_name.as_ref().map(|s| s.as_str()))
        .bind(test_item.day_scope)
        .execute(&**pool)
        .await
        .expect("Failed to insert business_day");

        // Insert the index record directly into database (triggers index cache notification)
        sqlx::query("INSERT INTO calendar_business_day_idx (id, country_id, country_subdivision_id, date_hash) VALUES ($1, $2, $3, $4)")
            .bind(item_idx.id)
            .bind(item_idx.country_id)
            .bind(item_idx.country_subdivision_id)
            .bind(item_idx.date_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert business_day index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        // Verify the INDEX cache was updated
        let idx_cache = business_day_repo.business_day_idx_cache.read().await;
        assert!(
            idx_cache.contains_primary(&item_idx.id),
            "BusinessDay should be in index cache after insert"
        );
        drop(idx_cache);

        // Verify the MAIN cache was updated
        let main_cache = business_day_repo.business_day_cache.read().await;
        assert!(
            main_cache.contains(&test_item.id),
            "BusinessDay should be in main cache after insert"
        );
        drop(main_cache);

        // Delete the record from database (triggers both cache notifications)
        sqlx::query("DELETE FROM calendar_business_day WHERE id = $1")
            .bind(test_item.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete business_day");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify removed from both caches
        let idx_cache = business_day_repo.business_day_idx_cache.read().await;
        assert!(
            !idx_cache.contains_primary(&item_idx.id),
            "BusinessDay should be removed from index cache after delete"
        );
        drop(idx_cache);

        let main_cache = business_day_repo.business_day_cache.read().await;
        assert!(
            !main_cache.contains(&test_item.id),
            "BusinessDay should be removed from main cache after delete"
        );

        Ok(())
    }
}