use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &ComplianceMetadataRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM compliance_metadata_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM compliance_metadata WHERE id = ANY($1)"#;

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
        
        // Update cache after releasing transaction lock
        drop(tx);
        {
            let cache = repo.compliance_metadata_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for ComplianceMetadataRepositoryImpl {
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
    use business_core_db::repository::load_batch::LoadBatch;
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let mut metadata_items = Vec::new();
        for i in 0..3 {
            let metadata = create_test_compliance_metadata(
                Some(&format!("DELETE-{}", i)),
                true,
                false,
            );
            metadata_items.push(metadata);
        }

        let saved_items = compliance_metadata_repo.create_batch(metadata_items.clone(), None).await?;
        let ids: Vec<uuid::Uuid> = saved_items.iter().map(|m| m.id).collect();

        let deleted_count = compliance_metadata_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, 3);

        // Verify items are deleted
        let loaded_items = compliance_metadata_repo.load_batch(&ids).await?;
        for item_opt in loaded_items {
            assert!(item_opt.is_none());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let deleted_count = compliance_metadata_repo.delete_batch(&[], None).await?;

        assert_eq!(deleted_count, 0);

        Ok(())
    }
}