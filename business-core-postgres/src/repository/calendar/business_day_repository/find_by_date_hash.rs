use std::error::Error;

use business_core_db::models::calendar::business_day::BusinessDayIdxModel;

use super::repo_impl::BusinessDayRepositoryImpl;

impl BusinessDayRepositoryImpl {
    pub async fn find_by_date_hash(
        &self,
        date_hash: i64,
    ) -> Result<Vec<BusinessDayIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.business_day_idx_cache.read().await;
        let items = cache.get_by_i64_index("date_hash", &date_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use chrono::NaiveDate;
    use super::super::test_utils::test_utils::create_test_business_day_with_date;

    #[tokio::test]
    async fn test_find_by_date_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let date_hash = test_date.and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_utc().timestamp() / 86400)
            .unwrap_or(0);
        
        let item1 = create_test_business_day_with_date(test_date);
        let item2 = create_test_business_day_with_date(test_date);
        let item3 = create_test_business_day_with_date(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap());
        
        let saved = business_day_repo.create_batch(vec![item1, item2, item3], None).await?;

        let found_items = business_day_repo.find_by_date_hash(date_hash).await?;
        
        assert_eq!(found_items.len(), 2);
        assert!(found_items.iter().all(|i| i.date_hash == date_hash));

        let non_existent_date_hash = 99999i64;
        let found_items = business_day_repo.find_by_date_hash(non_existent_date_hash).await?;
        assert!(found_items.is_empty());

        Ok(())
    }
}