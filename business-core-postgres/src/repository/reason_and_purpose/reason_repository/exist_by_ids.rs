use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &ReasonRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.reason_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for ReasonRepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
    use super::super::test_utils::test_utils::create_test_reason;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason(
                &format!("EXIST_TEST_{}", i),
                &format!("Exist Test Reason {}", i),
            ));
        }

        let saved = reason_repo.create_batch(reasons.clone(), None).await?;
        let ids: Vec<_> = saved.iter().map(|r| r.id).collect();

        let exists = reason_repo.exist_by_ids(&ids).await?;

        assert_eq!(exists.len(), 3);
        for (_, exists) in exists {
            assert!(exists);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_exist_by_ids_non_existent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let non_existent_id = uuid::Uuid::new_v4();
        let exists = reason_repo.exist_by_ids(&[non_existent_id]).await?;

        assert_eq!(exists.len(), 1);
        assert!(!exists[0].1);

        Ok(())
    }
}