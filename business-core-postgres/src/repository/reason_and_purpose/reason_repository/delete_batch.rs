use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &ReasonRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM reason_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM reason WHERE id = ANY($1)"#;

        let rows_affected = {
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
            result.rows_affected() as usize
        }; // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.reason_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for ReasonRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_reason;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason(
                &format!("DELETE_TEST_{i}"),
                &format!("Delete Test Reason {i}"),
            ));
        }

        let saved = reason_repo.create_batch(reasons.clone(), None).await?;
        let ids: Vec<_> = saved.iter().map(|r| r.id).collect();

        let deleted = reason_repo.delete_batch(&ids, None).await?;

        assert_eq!(deleted, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let deleted = reason_repo.delete_batch(&[], None).await?;

        assert_eq!(deleted, 0);

        Ok(())
    }
}