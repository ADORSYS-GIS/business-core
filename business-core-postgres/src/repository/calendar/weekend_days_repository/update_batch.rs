use async_trait::async_trait;
use business_core_db::models::calendar::weekend_days::WeekendDaysModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<WeekendDaysModel>,
    ) -> Result<Vec<WeekendDaysModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = self.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for item in items {
                // Execute update
                sqlx::query(
                    r#"
                    UPDATE calendar_weekend_days
                    SET country_id = $2, country_subdivision_id = $3,
                        weekend_day_01 = $4, weekend_day_02 = $5, weekend_day_03 = $6, weekend_day_04 = $7,
                        weekend_day_05 = $8, weekend_day_06 = $9, weekend_day_07 = $10,
                        effective_date = $11, expiry_date = $12
                    WHERE id = $1
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

                indices.push((item.id, item.to_index()));
                updated_items.push(item);
            }
        } // Transaction lock released here
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = self.weekend_days_idx_cache.read().await;
            let main_cache = self.weekend_days_cache.read().await;
            
            for (id, idx) in indices.iter() {
                idx_cache.remove(id);
                idx_cache.add(idx.clone());
                // Main cache update replaces the entire entity
                let item = updated_items.iter().find(|i| i.id == *id).unwrap();
                main_cache.update(item.clone());
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, WeekendDaysModel> for WeekendDaysRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<WeekendDaysModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<WeekendDaysModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use chrono::NaiveDate;
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let mut items = Vec::new();
        for _ in 0..3 {
            let item = create_test_weekend_days(None, None);
            items.push(item);
        }

        let saved_items = weekend_days_repo.create_batch(items.clone(), None).await?;
        
        let mut items_to_update = Vec::new();
        for mut item in saved_items {
            item.effective_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
            items_to_update.push(item);
        }

        let updated_items = weekend_days_repo.update_batch(items_to_update.clone(), None).await?;

        assert_eq!(updated_items.len(), 3);

        let ids: Vec<_> = updated_items.iter().map(|i| i.id).collect();
        let loaded = weekend_days_repo.load_batch(&ids).await?;
        
        for item_opt in loaded {
            let item = item_opt.unwrap();
            assert_eq!(item.effective_date, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        }

        // Verify updated entity in cache
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        let cached = main_cache.get(&updated_items[0].id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().effective_date, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let updated_items = weekend_days_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(updated_items.len(), 0);

        Ok(())
    }
}