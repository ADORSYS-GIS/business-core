use async_trait::async_trait;
use business_core_db::models::person::risk_summary::RiskSummaryModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<RiskSummaryModel>,
    ) -> Result<Vec<RiskSummaryModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute update
            sqlx::query(
                r#"
                UPDATE risk_summary
                SET person_id = $2, current_rating = $3, last_assessment_date = $4, flags_01 = $5, flags_02 = $6, flags_03 = $7, flags_04 = $8, flags_05 = $9
                WHERE id = $1
                "#,
            )
            .bind(item.id)
            .bind(item.person_id)
            .bind(item.current_rating)
            .bind(item.last_assessment_date)
            .bind(item.flags_01.as_str())
            .bind(item.flags_02.as_str())
            .bind(item.flags_03.as_str())
            .bind(item.flags_04.as_str())
            .bind(item.flags_05.as_str())
            .execute(&mut **transaction)
            .await?;

            // Update index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                UPDATE risk_summary_idx
                SET person_id = $2
                WHERE id = $1
                "#,
            )
            .bind(idx.id)
            .bind(idx.person_id)
            .execute(&mut **transaction)
            .await?;

            indices.push((item.id, idx));
            updated_items.push(item);
        }
        
        // Release transaction lock before updating cache
        drop(tx);
        
        // Update cache after releasing transaction lock
        {
            let cache = self.risk_summary_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, RiskSummaryModel> for RiskSummaryRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<RiskSummaryModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<RiskSummaryModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use business_core_db::models::person::common_enums::RiskRating;
    use super::super::test_utils::test_utils::{create_test_risk_summary, create_test_person};
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        // Create risk summary
        let risk_summary = create_test_risk_summary(person.id);
        let saved = risk_summary_repo.create_batch(vec![risk_summary.clone()], Some(audit_log.id)).await?;

        // Update the risk summary
        let mut updated_risk_summary = saved[0].clone();
        updated_risk_summary.current_rating = RiskRating::High;

        let updated = risk_summary_repo.update_batch(vec![updated_risk_summary.clone()], Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].id, saved[0].id);
        assert_eq!(updated[0].current_rating, RiskRating::High);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        let updated = risk_summary_repo.update_batch(vec![], None).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}