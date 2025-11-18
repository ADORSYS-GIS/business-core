use std::error::Error;

use business_core_db::models::calendar::weekend_days::WeekendDaysIdxModel;
use uuid::Uuid;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub async fn find_by_country_id(
        &self,
        country_id: Uuid,
    ) -> Result<Vec<WeekendDaysIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.weekend_days_idx_cache.read().await;
        let items = cache.get_by_uuid_index("country_id", &country_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_find_by_country_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let country_id = Uuid::new_v4();
        let item1 = create_test_weekend_days(Some(country_id), None);
        let item2 = create_test_weekend_days(Some(country_id), None);
        let item3 = create_test_weekend_days(None, None);
        
        let saved = weekend_days_repo.create_batch(vec![item1, item2, item3], None).await?;

        let found_items = weekend_days_repo.find_by_country_id(country_id).await?;
        
        assert_eq!(found_items.len(), 2);
        assert!(found_items.iter().all(|i| i.country_id == Some(country_id)));

        let non_existent_country_id = Uuid::new_v4();
        let found_items = weekend_days_repo.find_by_country_id(non_existent_country_id).await?;
        assert!(found_items.is_empty());

        Ok(())
    }
}