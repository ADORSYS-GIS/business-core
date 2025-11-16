use std::error::Error;
use business_core_db::models::reason_and_purpose::reason::ReasonIdxModel;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub async fn find_by_category_hash(
        &self,
        category_hash: i64,
    ) -> Result<Vec<ReasonIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.reason_idx_cache.read().await;
        let items = cache.get_by_i64_index("category_hash", &category_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use business_core_db::models::reason_and_purpose::reason::ReasonCategory;
    use super::super::test_utils::test_utils::create_test_reason_with_category;

    #[tokio::test]
    async fn test_find_by_category_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let test_category = ReasonCategory::Compliance;
        let expected_hash = hash_as_i64(&test_category.to_string()).unwrap();
        
        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason_with_category(
                &format!("CAT_TEST_{}", i),
                &format!("Test Reason {}", i),
                test_category,
            ));
        }

        let saved = reason_repo.create_batch(reasons, None).await?;

        let found = reason_repo.find_by_category_hash(expected_hash).await?;
        
        assert_eq!(found.len(), 3);
        for saved_reason in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_reason.id));
            assert!(found.iter().all(|idx| idx.category_hash == expected_hash));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_category_hash_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let non_existent_hash = hash_as_i64(&"NON_EXISTENT_CATEGORY").unwrap();
        let found = reason_repo.find_by_category_hash(non_existent_hash).await?;
        
        assert!(found.is_empty());

        Ok(())
    }
}