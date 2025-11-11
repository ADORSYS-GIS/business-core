use async_trait::async_trait;
use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &CountrySubdivisionRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<CountrySubdivisionModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM country_subdivision WHERE id = ANY($1)"#;
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
            let item = CountrySubdivisionModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, CountrySubdivisionModel> for CountrySubdivisionRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<CountrySubdivisionModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("IT", "Italy");
        let country_id = country.id;
        let audit_log_id = Uuid::new_v4();
        country_repo.create_batch(vec![country], audit_log_id).await?;

        let mut subdivisions = Vec::new();
        for i in 0..3 {
            let subdivision = create_test_country_subdivision(
                country_id,
                &format!("LD{}", i),
                &format!("Load Test Subdivision {}", i),
            );
            subdivisions.push(subdivision);
        }

        let audit_log_id = Uuid::new_v4();
        let saved = country_subdivision_repo.create_batch(subdivisions.clone(), audit_log_id).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = country_subdivision_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let subdivision = item.unwrap();
            assert_eq!(subdivision.country_id, country_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("ES", "Spain");
        let country_id = country.id;
        let audit_log_id = Uuid::new_v4();
        country_repo.create_batch(vec![country], audit_log_id).await?;

        let subdivision = create_test_country_subdivision(
            country_id,
            "NE1",
            "Non-Existing Test",
        );

        let audit_log_id = Uuid::new_v4();
        let saved = country_subdivision_repo.create_batch(vec![subdivision], audit_log_id).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = country_subdivision_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}