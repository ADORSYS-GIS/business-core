use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &RiskSummaryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.risk_summary_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for RiskSummaryRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_risk_summary;
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create risk summary
        let risk_summary = create_test_risk_summary();
        let saved = risk_summary_repo.create_batch(vec![risk_summary.clone()], Some(audit_log.id)).await?;

        // Check existence
        let non_existing_id = Uuid::new_v4();
        let ids = vec![saved[0].id, non_existing_id];
        let exists = risk_summary_repo.exist_by_ids(&ids).await?;

        assert_eq!(exists.len(), 2);
        assert_eq!(exists[0].0, saved[0].id);
        assert_eq!(exists[0].1, true);
        assert_eq!(exists[1].0, non_existing_id);
        assert_eq!(exists[1].1, false);

        Ok(())
    }
}