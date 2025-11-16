use std::error::Error;
use uuid::Uuid;
use business_core_db::models::reason_and_purpose::reason::ReasonIdxModel;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub async fn find_by_compliance_metadata(
        &self,
        compliance_metadata: Uuid,
    ) -> Result<Vec<ReasonIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.reason_idx_cache.read().await;
        let items = cache.get_by_uuid_index("compliance_metadata", &compliance_metadata);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use crate::repository::reason_and_purpose::compliance_metadata_repository::test_utils::test_utils::create_test_compliance_metadata;
    use super::super::test_utils::test_utils::create_test_reason_with_compliance_metadata;

    #[tokio::test]
    async fn test_find_by_compliance_metadata() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        // Create and save a compliance metadata record first
        let compliance_metadata = create_test_compliance_metadata(Some("REG-001"), true, false);
        let saved_compliance_metadata = compliance_metadata_repo.create_batch(vec![compliance_metadata], None).await?;
        let compliance_metadata_id = saved_compliance_metadata[0].id;
        
        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason_with_compliance_metadata(
                &format!("COMPLIANCE_TEST_{}", i),
                &format!("Test Reason {}", i),
                Some(compliance_metadata_id),
            ));
        }

        let saved = reason_repo.create_batch(reasons, None).await?;

        let found = reason_repo.find_by_compliance_metadata(compliance_metadata_id).await?;
        
        assert_eq!(found.len(), 3);
        for saved_reason in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_reason.id));
            assert!(found.iter().all(|idx| idx.compliance_metadata == Some(compliance_metadata_id)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_compliance_metadata_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let non_existent_id = uuid::Uuid::new_v4();
        let found = reason_repo.find_by_compliance_metadata(non_existent_id).await?;
        
        assert!(found.is_empty());

        Ok(())
    }
}