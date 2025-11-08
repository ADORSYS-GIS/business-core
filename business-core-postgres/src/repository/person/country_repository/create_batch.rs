use async_trait::async_trait;
use business_core_db::models::person::country::CountryModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &CountryRepositoryImpl,
        items: Vec<CountryModel>,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        
        for item in items {
            let query = sqlx::query(
                r#"
                INSERT INTO country (id, iso2, name_l1, name_l2, name_l3)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.iso2.as_str())
            .bind(item.name_l1.as_str())
            .bind(item.name_l2.as_ref().map(|s| s.as_str()))
            .bind(item.name_l3.as_ref().map(|s| s.as_str()));

            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.execute(&mut **transaction).await?;
            } else {
                return Err("Transaction has been consumed".into());
            }

            // Insert into index table
            let idx = item.to_index();
            let idx_query = sqlx::query(
                r#"
                INSERT INTO country_idx (id, iso2_hash)
                VALUES ($1, $2)
                "#,
            )
            .bind(idx.id)
            .bind(idx.iso2_hash);

            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                idx_query.execute(&mut **transaction).await?;
            } else {
                return Err("Transaction has been consumed".into());
            }

            // Update cache
            let mut cache = repo.country_idx_cache.write();
            cache.add(idx);
            drop(cache);

            saved_items.push(item);
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, CountryModel> for CountryRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<CountryModel>,
        _audit_log_id: Uuid,
    ) -> Result<Vec<CountryModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::repository::create_batch::CreateBatch;
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
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut countries = Vec::new();
        for i in 0..5 {
            let country = create_test_country(
                &format!("C{}", i),
                &format!("Test Country {}", i),
            );
            countries.push(country);
        }

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(countries.clone(), audit_log_id).await?;

        assert_eq!(saved_countries.len(), 5);

        for saved_country in &saved_countries {
            assert_eq!(saved_country.iso2.as_str().chars().next().unwrap(), 'C');
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let audit_log_id = Uuid::new_v4();
        let saved_countries = country_repo.create_batch(Vec::new(), audit_log_id).await?;

        assert_eq!(saved_countries.len(), 0);

        Ok(())
    }
}