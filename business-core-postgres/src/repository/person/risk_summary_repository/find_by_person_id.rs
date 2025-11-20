use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::risk_summary::RiskSummaryIdxModel;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub async fn find_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Vec<RiskSummaryIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.risk_summary_idx_cache.read().await;
        let items = cache.get_by_uuid_index("person_id", &person_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use super::super::test_utils::test_utils::{create_test_risk_summary, create_test_person};
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_find_by_person_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create person
        let person = create_test_person();
        person_repo.create_batch(vec![person.clone()], Some(audit_log.id)).await?;

        // Create test risk summaries with same person_id
        let mut risk_summaries = Vec::new();
        for _ in 0..3 {
            risk_summaries.push(create_test_risk_summary(person.id));
        }

        let saved = risk_summary_repo.create_batch(risk_summaries, Some(audit_log.id)).await?;

        // Find by person_id
        let found = risk_summary_repo.find_by_person_id(person.id).await?;
        
        assert_eq!(found.len(), 3);
        for saved_risk_summary in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_risk_summary.id));
            assert!(found.iter().all(|idx| idx.person_id == person.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_person_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        let non_existent_person_id = uuid::Uuid::new_v4();
        let found = risk_summary_repo.find_by_person_id(non_existent_person_id).await?;
        
        assert!(found.is_empty());

        Ok(())
    }
}