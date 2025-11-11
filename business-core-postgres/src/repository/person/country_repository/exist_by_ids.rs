use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &CountryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.country_idx_cache.read();
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for CountryRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let country1 = create_test_country("E1", "Test Country 1");
        let country2 = create_test_country("E2", "Test Country 2");

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(vec![country1.clone(), country2.clone()], audit_log_id).await?;

        let non_existent_id = Uuid::new_v4();
        let ids_to_check = vec![saved_countries[0].id, non_existent_id, saved_countries[1].id];
        let results = country_repo.exist_by_ids(&ids_to_check).await?;

        assert_eq!(results.len(), 3);

        let mut results_map = std::collections::HashMap::new();
        for (id, exists) in results {
            results_map.insert(id, exists);
        }

        assert_eq!(results_map.get(&saved_countries[0].id), Some(&true));
        assert_eq!(results_map.get(&saved_countries[1].id), Some(&true));
        assert_eq!(results_map.get(&non_existent_id), Some(&false));

        Ok(())
    }
}