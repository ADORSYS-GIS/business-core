use business_core_db::models::calendar::business_day::BusinessDayModel;
use business_core_db::repository::exist_by_ids::ExistByIds;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl ExistByIds for BusinessDayRepositoryImpl {
    type Model = BusinessDayModel;

    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &BusinessDayRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use index cache to check existence
        let idx_cache = repo.business_day_idx_cache.read().await;
        let existing_ids: Vec<Uuid> = ids
            .iter()
            .filter(|id| idx_cache.contains_primary(id))
            .copied()
            .collect();

        Ok(existing_ids)
    }
}