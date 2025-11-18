use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::WeekendDaysRepositoryImpl;

impl WeekendDaysRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &WeekendDaysRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM calendar_weekend_days_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM calendar_weekend_days WHERE id = ANY($1)"#;

        let rows_affected = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            sqlx::query(delete_idx_query).bind(ids).execute(&mut **transaction).await?;
            let result = sqlx::query(delete_query).bind(ids).execute(&mut **transaction).await?;
            result.rows_affected() as usize
        }; // Transaction lock released here
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.weekend_days_idx_cache.read().await;
            let main_cache = repo.weekend_days_cache.read().await;
            
            for id in ids {
                idx_cache.remove(id);
                main_cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for WeekendDaysRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_weekend_days;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let mut items = Vec::new();
        for _ in 0..3 {
            let item = create_test_weekend_days(None, None);
            items.push(item);
        }

        let saved_items = weekend_days_repo.create_batch(items.clone(), None).await?;
        let ids: Vec<Uuid> = saved_items.iter().map(|i| i.id).collect();

        let deleted_count = weekend_days_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, 3);

        // Verify removed from main cache
        let main_cache = weekend_days_repo.weekend_days_cache.read().await;
        for id in &ids {
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let weekend_days_repo = &ctx.calendar_repos().weekend_days_repository;

        let mut ids = Vec::new();
        for _ in 0..2 {
            ids.push(Uuid::new_v4());
        }

        let deleted_count = weekend_days_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, 0);

        Ok(())
    }
}