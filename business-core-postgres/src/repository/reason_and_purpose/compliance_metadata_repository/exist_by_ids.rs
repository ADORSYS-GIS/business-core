use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &ComplianceMetadataRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.compliance_metadata_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for ComplianceMetadataRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let mut metadata_items = Vec::new();
        for i in 0..3 {
            let metadata = create_test_compliance_metadata(
                Some(&format!("EXIST-{i}")),
                true,
                false,
            );
            metadata_items.push(metadata);
        }

        let saved_items = compliance_metadata_repo.create_batch(metadata_items.clone(), None).await?;
        let mut ids: Vec<Uuid> = saved_items.iter().map(|m| m.id).collect();
        
        // Add a non-existent ID
        let non_existent_id = Uuid::new_v4();
        ids.push(non_existent_id);

        let exist_results = compliance_metadata_repo.exist_by_ids(&ids).await?;
        assert_eq!(exist_results.len(), 4);

        // First 3 should exist
        for i in 0..3 {
            assert_eq!(exist_results[i].0, saved_items[i].id);
            assert!(exist_results[i].1);
        }

        // Last one should not exist
        assert_eq!(exist_results[3].0, non_existent_id);
        assert!(!exist_results[3].1);

        Ok(())
    }
}