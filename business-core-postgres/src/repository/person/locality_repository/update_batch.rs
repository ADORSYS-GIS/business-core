use async_trait::async_trait;
use business_core_db::models::person::locality::LocalityModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::LocalityRepositoryImpl;

impl LocalityRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<LocalityModel>,
    ) -> Result<Vec<LocalityModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = self.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for item in items {
                // Execute update
                sqlx::query(
                    r#"
                    UPDATE locality
                    SET country_subdivision_id = $2, code = $3, name = $4
                    WHERE id = $1
                    "#,
                )
                .bind(item.id)
                .bind(item.country_subdivision_id)
                .bind(item.code.as_str())
                .bind(item.name)
                .execute(&mut **transaction)
                .await?;

                indices.push((item.id, item.to_index()));
                updated_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = self.locality_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, LocalityModel> for LocalityRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<LocalityModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<LocalityModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use uuid::Uuid;
    use crate::repository::person::test_utils::{create_test_country, create_test_country_subdivision, create_test_locality};

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("JP", "Japan");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "TK", "Tokyo");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], None).await?;

        let mut localities = Vec::new();
        for i in 0..3 {
            let locality = create_test_locality(
                subdivision_id,
                &format!("UPD{i}"),
                &format!("Update Test {i}"),
            );
            localities.push(locality);
        }

        let saved = locality_repo.create_batch(localities, None).await?;

        // Update localities
        let mut updated_localities = Vec::new();
        for mut locality in saved {
            locality.name = Uuid::new_v4(); // Updated name reference
            updated_localities.push(locality);
        }

        let updated = locality_repo.update_batch(updated_localities, None).await?;

        assert_eq!(updated.len(), 3);
        for locality in updated {
            // Verify that update was successful
            assert_eq!(locality.country_subdivision_id, subdivision_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let locality_repo = &ctx.person_repos().locality_repository;

        let updated = locality_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}