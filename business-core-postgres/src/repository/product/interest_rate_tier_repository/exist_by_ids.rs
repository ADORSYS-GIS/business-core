use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &InterestRateTierRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut missing_ids = Vec::new();

        {
            let cache = repo.interest_rate_tier_idx_cache.read().await;
            for &id in ids {
                if cache.contains_primary(&id) {
                    results.push((id, true));
                } else {
                    missing_ids.push(id);
                }
            }
        }

        if missing_ids.is_empty() {
            return Ok(results);
        }

        // Check database for missing IDs
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

        let query = format!(
            "SELECT id FROM interest_rate_tier_idx WHERE id IN ({})",
            missing_ids
                .iter()
                .map(|id| format!("'{}'", id))
                .collect::<Vec<String>>()
                .join(",")
        );

        let found_ids: Vec<(Uuid,)> = sqlx::query_as(&query)
            .fetch_all(&mut **transaction)
            .await?;

        let found_ids_set: std::collections::HashSet<Uuid> =
            found_ids.into_iter().map(|(id,)| id).collect();

        for id in missing_ids {
            results.push((id, found_ids_set.contains(&id)));
        }

        Ok(results)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for InterestRateTierRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::setup_test_context;
    use business_core_db::{
        repository::{create_batch::CreateBatch, exist_by_ids::ExistByIds},
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut interest_rate_tiers = Vec::new();
        for _ in 0..3 {
            let interest_rate_tier = create_test_interest_rate_tier();
            interest_rate_tiers.push(interest_rate_tier);
        }

        let saved = interest_rate_tier_repo
            .create_batch(interest_rate_tiers, Some(audit_log.id))
            .await?;

        let mut ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let non_existent_id = Uuid::new_v4();
        ids.push(non_existent_id); // Add a non-existent ID

        let exists = interest_rate_tier_repo.exist_by_ids(&ids).await?;

        assert_eq!(exists.len(), 4);

        for (id, exists_flag) in exists {
            if id == non_existent_id {
                assert!(!exists_flag);
            } else {
                assert!(exists_flag);
            }
        }

        Ok(())
    }
}