use std::error::Error;
use business_core_db::models::reason_and_purpose::compliance_metadata::ComplianceMetadataIdxModel;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub async fn find_by_regulatory_code_hash(
        &self,
        regulatory_code_hash: i64,
    ) -> Result<Vec<ComplianceMetadataIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.compliance_metadata_idx_cache.read().await;
        let items = cache.get_by_i64_index("regulatory_code_hash", &regulatory_code_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_find_by_regulatory_code_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let test_code = "FATF-R.16";
        let mut metadata_items = Vec::new();
        for _i in 0..3 {
            let metadata = create_test_compliance_metadata(Some(test_code), true, false);
            metadata_items.push(metadata);
        }

        let saved = compliance_metadata_repo.create_batch(metadata_items, None).await?;

        let expected_hash = hash_as_i64(&test_code)?;
        let found = compliance_metadata_repo.find_by_regulatory_code_hash(expected_hash).await?;
        
        assert_eq!(found.len(), 3);
        for saved_item in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_item.id));
            assert!(found.iter().all(|idx| idx.regulatory_code_hash == Some(expected_hash)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_regulatory_code_hash_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let non_existent_hash = hash_as_i64(&"NON-EXISTENT")?;
        let found = compliance_metadata_repo.find_by_regulatory_code_hash(non_existent_hash).await?;
        
        assert!(found.is_empty());

        Ok(())
    }
}