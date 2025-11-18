use async_trait::async_trait;
use business_core_db::models::calendar::weekend_days::WeekendDaysModel;
use business_core_db::repository::load_batch::LoadBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use crate::utils::TryFromRow;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &WeekendDaysRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<WeekendDaysModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try to get from cache first
        let main_cache = repo.weekend_days_cache.read().await;
        let mut result = Vec::with_capacity(ids.len());
        let mut missing_ids = Vec::new();
        
        for &id in ids {
            match main_cache.get(&id) {
                Some(item) => result.push(Some(item)),
                None => {
                    result.push(None);
                    missing_ids.push(id);
                }
            }
        }
        
        drop(main_cache); // Release read lock
        
        // If all found in cache, return early
        if missing_ids.is_empty() {
            return Ok(result);
        }
        
        // Load missing items from database
        let query = r#"SELECT * FROM calendar_weekend_days WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(&missing_ids).fetch_all(&mut **transaction).await?
        };
        
        let mut loaded_map = std::collections::HashMap::new();
        for row in rows {
            let item = WeekendDaysModel::try_from_row(&row)?;
            loaded_map.insert(item.id, item);
        }
        
        // Update result and add to cache
        let main_cache = repo.weekend_days_cache.read().await;
        for (i, &id) in ids.iter().enumerate() {
            if result[i].is_none() {
                if let Some(item) = loaded_map.remove(&id) {
                    main_cache.insert(item.clone());
                    result[i] = Some(item);
                }
            }
        }
        
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, WeekendDaysModel> for WeekendDaysRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<WeekendDaysModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let mut items = Vec::new();
        for _ in 0..3 {
            let item = create_test_weekend_days(None, None);
            items.push(item);
        }

        let saved_items = weekend_days_repo.create_batch(items.clone(), None).await?;
        let ids: Vec<_> = saved_items.iter().map(|i| i.id).collect();

        let loaded_items = weekend_days_repo.load_batch(&ids).await?;

        assert_eq!(loaded_items.len(), 3);
        for item in loaded_items {
            assert!(item.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let items = vec![create_test_weekend_days(None, None)];
        let saved = weekend_days_repo.create_batch(items, None).await?;
        let ids: Vec<_> = saved.iter().map(|i| i.id).collect();

        // First load - should populate cache
        let loaded1 = weekend_days_repo.load_batch(&ids).await?;
        
        // Second load - should hit cache
        let loaded2 = weekend_days_repo.load_batch(&ids).await?;
        
        assert_eq!(loaded1.len(), loaded2.len());
        
        // Verify cache statistics
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        let stats = main_cache.statistics();
        assert!(stats.hits() > 0, "Should have cache hits on second load");

        Ok(())
    }
}