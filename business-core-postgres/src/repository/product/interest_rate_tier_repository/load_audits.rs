use async_trait::async_trait;
use business_core_db::models::product::interest_rate_tier::InterestRateTierModel;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::pagination::{Page, PageRequest};
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn load_audits_impl(
        repo: &InterestRateTierRepositoryImpl,
        id: Uuid,
        page_request: PageRequest,
    ) -> Result<Page<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM interest_rate_tier_audit
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut **transaction)
        .await?;

        let items = sqlx::query_as::<_, InterestRateTierModel>(
            r#"
            SELECT * FROM interest_rate_tier_audit
            WHERE id = $1
            ORDER BY antecedent_audit_log_id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(id)
        .bind(page_request.limit as i64)
        .bind(page_request.offset as i64)
        .fetch_all(&mut **transaction)
        .await?;

        Ok(Page::new(items, total_count as usize, page_request.limit, page_request.offset))
    }
}

#[async_trait]
impl LoadAudits<Postgres, InterestRateTierModel> for InterestRateTierRepositoryImpl {
    async fn load_audits(
        &self,
        id: Uuid,
        page_request: PageRequest,
    ) -> Result<Page<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        Self::load_audits_impl(self, id, page_request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::setup_test_context;
    use business_core_db::{
        repository::{create_batch::CreateBatch, load_audits::LoadAudits, update_batch::UpdateBatch},
        repository::pagination::PageRequest,
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        // Create initial entity
        let interest_rate_tier = create_test_interest_rate_tier();
        let interest_rate_tier_id = interest_rate_tier.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let mut saved = interest_rate_tier_repo
            .create_batch(vec![interest_rate_tier.clone()], Some(audit_log.id))
            .await?;

        // Update the entity multiple times to create audit history
        for i in 1..=3 {
            let audit_log = create_test_audit_log();
            audit_log_repo.create(&audit_log).await?;

            let mut updated = saved[0].clone();
            updated.interest_rate = Decimal::from(i);
            saved = interest_rate_tier_repo
                .update_batch(vec![updated], Some(audit_log.id))
                .await?;
        }

        // Load first page of audit records
        let page = interest_rate_tier_repo
            .load_audits(interest_rate_tier_id, PageRequest::new(2, 0))
            .await?;

        // Should have 4 total audit records (1 create + 3 updates)
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = interest_rate_tier_repo
            .load_audits(interest_rate_tier_id, PageRequest::new(2, 2))
            .await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2); // Second page with remaining 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        // Try to load audits for non-existing entity
        let non_existing_id = Uuid::new_v4();
        let page = interest_rate_tier_repo
            .load_audits(non_existing_id, PageRequest::new(20, 0))
            .await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}