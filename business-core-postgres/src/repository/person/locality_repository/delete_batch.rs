use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::LocalityRepositoryImpl;

impl LocalityRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &LocalityRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM locality_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM locality WHERE id = ANY($1)"#;

        let rows_affected = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            sqlx::query(delete_idx_query).bind(ids).execute(&mut **transaction).await?;
            let result = sqlx::query(delete_query).bind(ids).execute(&mut **transaction).await?;
            result.rows_affected() as usize
        }; // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.locality_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for LocalityRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        _audit_log_id: Uuid,
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
    use super::super::test_utils::{create_test_country, create_test_country_subdivision, create_test_locality};

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        let audit_log_id = Uuid::new_v4();
        country_repo.create_batch(vec![country], audit_log_id).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "CA", "California");
        let subdivision_id = subdivision.id;
        let audit_log_id = Uuid::new_v4();
        country_subdivision_repo.create_batch(vec![subdivision], audit_log_id).await?;

        let mut localities = Vec::new();
        for i in 0..3 {
            let locality = create_test_locality(
                subdivision_id,
                &format!("DEL{}", i),
                &format!("Delete Test {}", i),
            );
            localities.push(locality);
        }

        let audit_log_id = Uuid::new_v4();
        let saved = locality_repo.create_batch(localities, audit_log_id).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let audit_log_id = Uuid::new_v4();
        let deleted_count = locality_repo.delete_batch(&ids, audit_log_id).await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("CA", "Canada");
        let country_id = country.id;
        let audit_log_id = Uuid::new_v4();
        country_repo.create_batch(vec![country], audit_log_id).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "ON", "Ontario");
        let subdivision_id = subdivision.id;
        let audit_log_id = Uuid::new_v4();
        country_subdivision_repo.create_batch(vec![subdivision], audit_log_id).await?;

        let locality = create_test_locality(
            subdivision_id,
            "DELN",
            "Delete Non-Existing Test",
        );

        let audit_log_id = Uuid::new_v4();
        let saved = locality_repo.create_batch(vec![locality], audit_log_id).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let audit_log_id = Uuid::new_v4();
        let deleted_count = locality_repo.delete_batch(&ids, audit_log_id).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}