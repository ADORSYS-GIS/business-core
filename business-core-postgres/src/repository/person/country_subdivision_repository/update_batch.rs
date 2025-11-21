use async_trait::async_trait;
use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<CountrySubdivisionModel>,
    ) -> Result<Vec<CountrySubdivisionModel>, Box<dyn Error + Send + Sync>> {
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
                    UPDATE country_subdivision
                    SET country_id = $2, code = $3, name = $4
                    WHERE id = $1
                    "#,
                )
                .bind(item.id)
                .bind(item.country_id)
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
            let cache = self.country_subdivision_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, CountrySubdivisionModel> for CountrySubdivisionRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<CountrySubdivisionModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<CountrySubdivisionModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("JP", "Japan");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        let mut subdivisions = Vec::new();
        for i in 0..3 {
            let subdivision = create_test_country_subdivision(
                country_id,
                &format!("UPD{i}"),
                &format!("Update Test {i}"),
            );
            subdivisions.push(subdivision);
        }

        let saved = country_subdivision_repo.create_batch(subdivisions, None).await?;

        // Update subdivisions
        let mut updated_subdivisions = Vec::new();
        for mut subdivision in saved {
            subdivision.name = Uuid::new_v4(); // Update to a new name UUID
            updated_subdivisions.push(subdivision);
        }

        let updated = country_subdivision_repo.update_batch(updated_subdivisions, None).await?;

        assert_eq!(updated.len(), 3);
        for subdivision in updated {
            // Verify update was successful - just check that items were returned
            assert_eq!(subdivision.country_id, country_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        let updated = country_subdivision_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}