use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &CountrySubdivisionRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM country_subdivision_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM country_subdivision WHERE id = ANY($1)"#;

        let rows_affected = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            sqlx::query(delete_idx_query).bind(ids).execute(&mut **transaction).await?;
            let result = sqlx::query(delete_query).bind(ids).execute(&mut **transaction).await?;
            result.rows_affected() as usize
        }; // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.country_subdivision_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for CountrySubdivisionRepositoryImpl {
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
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        let mut subdivisions = Vec::new();
        for i in 0..3 {
            let subdivision = create_test_country_subdivision(
                country_id,
                &format!("DEL{}", i),
                &format!("Delete Test {}", i),
            );
            subdivisions.push(subdivision);
        }

        let saved = country_subdivision_repo.create_batch(subdivisions, None).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let deleted_count = country_subdivision_repo.delete_batch(&ids, None).await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("CA", "Canada");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        let subdivision = create_test_country_subdivision(
            country_id,
            "DELN",
            "Delete Non-Existing Test",
        );

        let saved = country_subdivision_repo.create_batch(vec![subdivision], None).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let deleted_count = country_subdivision_repo.delete_batch(&ids, None).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}