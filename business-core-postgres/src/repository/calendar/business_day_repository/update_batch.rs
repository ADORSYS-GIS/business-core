use business_core_db::models::calendar::business_day::BusinessDayModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl UpdateBatch<sqlx::Postgres, BusinessDayModel> for BusinessDayRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<BusinessDayModel>,
        _audit_info: Option<Uuid>,
    ) -> Result<Vec<BusinessDayModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn update_batch_impl(
        repo: &BusinessDayRepositoryImpl,
        items: Vec<BusinessDayModel>,
    ) -> Result<Vec<BusinessDayModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute update on main table
            sqlx::query(
                r#"
                UPDATE calendar_business_day
                SET country_id = $2, country_subdivision_id = $3, date = $4, weekday = $5,
                    is_business_day = $6, is_weekend = $7, weekend_day_01 = $8, is_holiday = $9,
                    holiday_name = $10, day_scope = $11
                WHERE id = $1
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
            
            // Update index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                UPDATE calendar_business_day_idx
                SET country_id = $2, country_subdivision_id = $3, date_hash = $4
                WHERE id = $1
                "#,
            )
            .bind(idx.id)
            .bind(idx.country_id)
            .bind(idx.country_subdivision_id)
            .bind(idx.date_hash)
            .execute(&mut **transaction)
            .await?;
            
            indices.push((item.id, idx));
            updated_items.push(item);
        }
        
        drop(tx); // Release transaction lock
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.business_day_idx_cache.read().await;
            let main_cache = repo.business_day_cache.read().await;
            
            for (id, idx) in indices.iter() {
                idx_cache.remove(id);
                idx_cache.add(idx.clone());
                // Main cache update replaces the entire entity
                if let Some(item) = updated_items.iter().find(|i| i.id == *id) {
                    main_cache.update(item.clone());
                }
            }
        }

        Ok(updated_items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_business_day;

    #[tokio::test]
    async fn test_update_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        let items = vec![create_test_business_day(Some(Uuid::new_v4()), None)];
        let mut saved = business_day_repo.create_batch(items, None).await?;
        
        saved[0].is_business_day = false;
        saved[0].is_holiday = true;
        let updated = business_day_repo.update_batch(saved, None).await?;

        // Verify updated entity in cache
        let main_cache = business_day_repo.business_day_cache.read().await;
        let cached = main_cache.get(&updated[0].id);
        assert!(cached.is_some());
        let cached_ref = cached.as_ref().unwrap();
        assert_eq!(cached_ref.is_business_day, updated[0].is_business_day);
        assert_eq!(cached_ref.is_holiday, updated[0].is_holiday);

        Ok(())
    }
}