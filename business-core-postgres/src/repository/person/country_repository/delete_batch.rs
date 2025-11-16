use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &CountryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM country_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM country WHERE id = ANY($1)"#;

        let rows_affected = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            sqlx::query(delete_idx_query).bind(ids).execute(&mut **transaction).await?;
            let result = sqlx::query(delete_query).bind(ids).execute(&mut **transaction).await?;
            result.rows_affected() as usize
        }; // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.country_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for CountryRepositoryImpl {
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
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("D{}", i),
                &format!("Test Country {}", i),
            );
            countries.push(country);
        }

        let saved_countries = country_repo.create_batch(countries.clone(), None).await?;
        let ids: Vec<Uuid> = saved_countries.iter().map(|c| c.id).collect();

        let deleted_count = country_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut ids = Vec::new();
        for _ in 0..2 {
            ids.push(Uuid::new_v4());
        }

        let deleted_count = country_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, 0);

        Ok(())
    }
}