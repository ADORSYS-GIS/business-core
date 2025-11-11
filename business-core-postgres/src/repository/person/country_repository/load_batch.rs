use async_trait::async_trait;
use business_core_db::models::person::country::CountryModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &CountryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<CountryModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM country WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = CountryModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, CountryModel> for CountryRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<CountryModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("L{}", i),
                &format!("Test Country {}", i),
            );
            countries.push(country);
        }

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(countries.clone(), audit_log_id).await?;
        let ids: Vec<Uuid> = saved_countries.iter().map(|c| c.id).collect();

        let loaded_countries = country_repo.load_batch(&ids).await?;
        assert_eq!(loaded_countries.len(), 5);
        
        for country_opt in loaded_countries {
            assert!(country_opt.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let country = create_test_country("L9", "Test Country");
        let audit_log_id = Uuid::new_v4();
        let saved = country_repo.create_batch(vec![country.clone()], audit_log_id).await?;

        let non_existent_id = Uuid::new_v4();
        let ids = vec![saved[0].id, non_existent_id];

        let loaded_countries = country_repo.load_batch(&ids).await?;
        assert_eq!(loaded_countries.len(), 2);
        assert!(loaded_countries[0].is_some());
        assert!(loaded_countries[1].is_none());

        Ok(())
    }
}