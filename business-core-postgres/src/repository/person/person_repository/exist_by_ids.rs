use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &PersonRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.person_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for PersonRepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
    use uuid::Uuid;
    use business_core_db::models::person::person::PersonType;
    use crate::repository::person::person_repository::test_utils::create_test_person;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let person = create_test_person(
            "Existing Person",
            PersonType::Natural,
        );

        let saved = person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let existing_id = saved[0].id;
        let non_existing_id = Uuid::new_v4();

        let result = person_repo.exist_by_ids(&[existing_id, non_existing_id]).await?;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (existing_id, true));
        assert_eq!(result[1], (non_existing_id, false));

        Ok(())
    }
}