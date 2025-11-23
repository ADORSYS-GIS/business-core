use async_trait::async_trait;
use business_core_db::models::product::interest_rate_tier::InterestRateTierModel;
use business_core_db::repository::load_batch::LoadBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &InterestRateTierRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<InterestRateTierModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

        let items: Vec<InterestRateTierModel> = sqlx::query_as(
            r#"
            SELECT * FROM interest_rate_tier
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await?;

        let items_map: std::collections::HashMap<Uuid, InterestRateTierModel> =
            items.into_iter().map(|item| (item.id, item)).collect();
        let results = ids
            .iter()
            .map(|id| items_map.get(id).cloned())
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl LoadBatch<Postgres, InterestRateTierModel> for InterestRateTierRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<InterestRateTierModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::setup_test_context;
    use business_core_db::{
        repository::{create_batch::CreateBatch, load_batch::LoadBatch},
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = interest_rate_tier_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for loaded_entity in loaded {
            let entity = loaded_entity.expect("loaded entity should not be None");
            assert!(ids.contains(&entity.id));
        }

        Ok(())
    }
}