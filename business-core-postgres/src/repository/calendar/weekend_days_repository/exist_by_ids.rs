use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &WeekendDaysRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.weekend_days_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for WeekendDaysRepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let item1 = create_test_weekend_days(None, None);
        let item2 = create_test_weekend_days(None, None);

        let saved_items = weekend_days_repo.create_batch(vec![item1.clone(), item2.clone()], None).await?;

        let non_existent_id = Uuid::new_v4();
        let ids_to_check = vec![saved_items[0].id, non_existent_id, saved_items[1].id];
        let results = weekend_days_repo.exist_by_ids(&ids_to_check).await?;

        assert_eq!(results.len(), 3);

        let mut results_map = std::collections::HashMap::new();
        for (id, exists) in results {
            results_map.insert(id, exists);
        }

        assert_eq!(results_map.get(&saved_items[0].id), Some(&true));
        assert_eq!(results_map.get(&saved_items[1].id), Some(&true));
        assert_eq!(results_map.get(&non_existent_id), Some(&false));

        Ok(())
    }
}