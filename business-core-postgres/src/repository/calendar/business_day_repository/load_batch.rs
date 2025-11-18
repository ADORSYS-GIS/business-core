use business_core_db::models::calendar::business_day::BusinessDayModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl LoadBatch<sqlx::Postgres, BusinessDayModel> for BusinessDayRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<BusinessDayModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &BusinessDayRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<BusinessDayModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try to get from cache first
        let main_cache = repo.business_day_cache.read().await;
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
        let query = r#"SELECT * FROM calendar_business_day WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(&missing_ids).fetch_all(&mut **transaction).await?
        };
        
        let mut loaded_map = std::collections::HashMap::new();
        for row in rows {
            let item = BusinessDayModel::try_from_row(&row)?;
            loaded_map.insert(item.id, item);
        }
        
        // Update result and add to cache
        let main_cache = repo.business_day_cache.read().await;
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

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_business_day;

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        let items = vec![
            create_test_business_day(Some(Uuid::new_v4()), None),
            create_test_business_day(None, Some(Uuid::new_v4())),
        ];
        let saved = business_day_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // First load - should populate cache
        let loaded1 = business_day_repo.load_batch(&ids).await?;
        
        // Second load - should hit cache
        let loaded2 = business_day_repo.load_batch(&ids).await?;
        
        assert_eq!(loaded1.len(), loaded2.len());
        assert_eq!(loaded1.len(), 2);
        for item in &loaded2 {
            assert!(item.is_some());
        }

        Ok(())
    }
}