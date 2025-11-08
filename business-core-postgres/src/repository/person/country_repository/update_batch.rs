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
        
        for item in items {
            let query = sqlx::query(
                r#"
                UPDATE country
                SET iso2 = $2, name_l1 = $3, name_l2 = $4, name_l3 = $5
                WHERE id = $1
                "#,
            )
            .bind(item.id)
            .bind(item.iso2.as_str())
            .bind(item.name_l1.as_str())
            .bind(item.name_l2.as_ref().map(|s| s.as_str()))
            .bind(item.name_l3.as_ref().map(|s| s.as_str()));

            let mut tx = self.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.execute(&mut **transaction).await?;
            } else {
                return Err("Transaction has been consumed".into());
            }

            // Update index cache
            let idx = item.to_index();
            let mut cache = self.country_idx_cache.write();
            cache.remove(&item.id);
            cache.add(idx);
            drop(cache);

            updated_items.push(item);
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, CountryModel> for CountryRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<CountryModel>,
        _audit_log_id: Uuid,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    fn create_test_country(iso2: &str, name: &str) -> CountryModel {
        CountryModel {
            id: Uuid::new_v4(),
            iso2: HeaplessString::try_from(iso2).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("U{}", i),
                &format!("Test Country {}", i),
            );
            countries.push(country);
        }

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(countries.clone(), audit_log_id).await?;
        
        let mut countries_to_update = Vec::new();
        for mut country in saved_countries {
            country.name_l1 = HeaplessString::try_from("Updated Name").unwrap();
            countries_to_update.push(country);
        }

        let updated_countries = country_repo.update_batch(countries_to_update.clone(), audit_log_id).await?;

        assert_eq!(updated_countries.len(), 5);

        let ids: Vec<Uuid> = updated_countries.iter().map(|c| c.id).collect();
        let loaded = country_repo.load_batch(&ids).await?;
        
        for country_opt in loaded {
            let country = country_opt.unwrap();
            assert_eq!(country.name_l1.as_str(), "Updated Name");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let audit_log_id = Uuid::new_v4();
        let updated_countries = country_repo.update_batch(Vec::new(), audit_log_id).await?;

        assert_eq!(updated_countries.len(), 0);

        Ok(())
    }
}