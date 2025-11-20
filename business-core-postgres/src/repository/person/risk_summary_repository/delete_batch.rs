use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &RiskSummaryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM risk_summary_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM risk_summary WHERE id = ANY($1)"#;

        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        sqlx::query(delete_idx_query)
            .bind(ids)
            .execute(&mut **transaction)
            .await?;
        let result = sqlx::query(delete_query)
            .bind(ids)
            .execute(&mut **transaction)
            .await?;
        let rows_affected = result.rows_affected() as usize;
        
        // Release transaction lock before updating cache
        drop(tx);
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.risk_summary_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for RiskSummaryRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        _audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_risk_summary, create_test_person};
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        // Create risk summaries
        let risk_summary1 = create_test_risk_summary(person.id);
        let risk_summary2 = create_test_risk_summary(person.id);

        let saved = risk_summary_repo.create_batch(vec![risk_summary1.clone(), risk_summary2.clone()], Some(audit_log.id)).await?;

        // Delete the risk summaries
        let ids = vec![saved[0].id, saved[1].id];
        let deleted = risk_summary_repo.delete_batch(&ids, Some(audit_log.id)).await?;

        assert_eq!(deleted, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing_risk_summaries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Try to delete non-existing risk summaries
        let non_existing_id1 = Uuid::new_v4();
        let non_existing_id2 = Uuid::new_v4();
        let ids = vec![non_existing_id1, non_existing_id2];
        let deleted = risk_summary_repo.delete_batch(&ids, None).await?;

        assert_eq!(deleted, 0);

        Ok(())
    }
}