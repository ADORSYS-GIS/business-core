use std::error::Error;
use uuid::Uuid;

use super::repo_impl::LocalityRepositoryImpl;

impl LocalityRepositoryImpl {
    pub async fn find_ids_by_country_subdivision_id(
        &self,
        country_subdivision_id: Uuid,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.locality_idx_cache.read().await;
        let items = cache.get_by_uuid_index("country_subdivision_id", &country_subdivision_id);
        let result = items.into_iter().map(|item| item.id).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;
    use crate::repository::person::test_utils::{create_test_country, create_test_country_subdivision, create_test_locality};

    #[tokio::test]
    async fn test_find_ids_by_country_subdivision_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("DE", "Germany");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "BE", "Berlin");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], None).await?;

        let mut localities = Vec::new();
        for i in 0..3 {
            let locality = create_test_locality(
                subdivision_id,
                &format!("CID{}", i),
                &format!("Subdivision ID Test {}", i),
            );
            localities.push(locality);
        }

        let saved = locality_repo.create_batch(localities, None).await?;

        let found_ids = locality_repo.find_ids_by_country_subdivision_id(subdivision_id).await?;
        
        assert_eq!(found_ids.len(), 3);
        for saved_locality in &saved {
            assert!(found_ids.contains(&saved_locality.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_ids_by_country_subdivision_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let locality_repo = &ctx.person_repos().locality_repository;

        let non_existent_subdivision_id = Uuid::new_v4();
        let found_ids = locality_repo.find_ids_by_country_subdivision_id(non_existent_subdivision_id).await?;
        
        assert!(found_ids.is_empty());

        Ok(())
    }
}