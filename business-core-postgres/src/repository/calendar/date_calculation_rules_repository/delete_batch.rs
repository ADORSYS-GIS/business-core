use business_core_db::repository::delete_batch::DeleteBatch;
use super::repo_impl::DateCalculationRulesRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl DeleteBatch<sqlx::Postgres> for DateCalculationRulesRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        _audit_info: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids).await
    }
}

impl DateCalculationRulesRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &DateCalculationRulesRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM calendar_date_calculation_rules_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM calendar_date_calculation_rules WHERE id = ANY($1)"#;

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
            let idx_cache = repo.date_calculation_rules_idx_cache.read().await;
            let main_cache = repo.date_calculation_rules_cache.read().await;
            
            for id in ids {
                idx_cache.remove(id);
                main_cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}