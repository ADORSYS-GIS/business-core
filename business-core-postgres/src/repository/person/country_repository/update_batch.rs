use async_trait::async_trait;
use business_core_db::models::person::country::CountryModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<CountryModel>,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
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
                    UPDATE country
                    SET iso2 = $2, name = $3
                    WHERE id = $1
                    "#,
                )
                .bind(item.id)
                .bind(item.iso2.as_str())
                .bind(item.name)
                .execute(&mut **transaction)
                .await?;

                indices.push((item.id, item.to_index()));
                updated_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = self.country_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, CountryModel> for CountryRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<CountryModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("U{i}"),
                &format!("Test Country {i}"),
            );
            countries.push(country);
        }

        let saved_countries = country_repo.create_batch(countries.clone(), None).await?;
        
        let mut countries_to_update = Vec::new();
        for mut country in saved_countries {
            country.name = Uuid::new_v4();
            countries_to_update.push(country);
        }

        let updated_countries = country_repo.update_batch(countries_to_update.clone(), None).await?;

        assert_eq!(updated_countries.len(), 5);

        let ids: Vec<Uuid> = updated_countries.iter().map(|c| c.id).collect();
        let loaded = country_repo.load_batch(&ids).await?;
        
        for country_opt in loaded {
            let country = country_opt.unwrap();
            // Verify country was loaded successfully
            assert!(countries_to_update.iter().any(|c| c.id == country.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let updated_countries = country_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(updated_countries.len(), 0);

        Ok(())
    }
}