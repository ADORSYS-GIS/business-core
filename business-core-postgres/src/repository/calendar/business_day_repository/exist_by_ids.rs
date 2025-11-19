use business_core_db::repository::exist_by_ids::ExistByIds;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl ExistByIds<sqlx::Postgres> for BusinessDayRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &BusinessDayRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use index cache to check existence
        let idx_cache = repo.business_day_idx_cache.read().await;
        let mut result = Vec::new();
        for &id in ids {
            result.push((id, idx_cache.contains_primary(&id)));
        }

        Ok(result)
    }
}