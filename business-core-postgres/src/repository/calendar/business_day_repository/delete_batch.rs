use business_core_db::models::calendar::business_day::BusinessDayModel;
use business_core_db::repository::delete_batch::DeleteBatch;
use super::repo_impl::BusinessDayRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;
use sqlx::Postgres;

#[async_trait]
impl DeleteBatch<sqlx::Postgres> for BusinessDayRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        _audit_info: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids).await
    }
}

impl BusinessDayRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &BusinessDayRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM calendar_business_day_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM calendar_business_day WHERE id = ANY($1)"#;

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
        
        drop(tx); // Release transaction lock
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.business_day_idx_cache.read().await;
            let main_cache = repo.business_day_cache.read().await;
            
            for id in ids {
                idx_cache.remove(id);
                main_cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_business_day;

    #[tokio::test]
    async fn test_delete_batch_removes_from_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let business_day_repo = &ctx.calendar_repos().business_day_repository;

        let items = vec![
            create_test_business_day(Some(Uuid::new_v4()), None),
            create_test_business_day(None, Some(Uuid::new_v4())),
        ];
        let saved = business_day_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // Delete entities
        let deleted_count = business_day_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, ids.len());

        // Verify removed from main cache
        let main_cache = business_day_repo.business_day_cache.read().await;
        for id in &ids {
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }
}