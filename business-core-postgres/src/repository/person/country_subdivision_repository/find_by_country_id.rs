use std::error::Error;
use uuid::Uuid;

use business_core_db::models::person::country_subdivision::CountrySubdivisionIdxModel;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub async fn find_by_country_id(
        &self,
        country_id: Uuid,
    ) -> Result<Vec<CountrySubdivisionIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.country_subdivision_idx_cache.read().await;
        let items = cache.get_by_uuid_index("country_id", &country_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_find_by_country_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("DE", "Germany");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        let mut subdivisions = Vec::new();
        for i in 0..3 {
            let subdivision = create_test_country_subdivision(
                country_id,
                &format!("CID{}", i),
                &format!("Country ID Test {}", i),
            );
            subdivisions.push(subdivision);
        }

        let saved = country_subdivision_repo.create_batch(subdivisions, None).await?;

        let found_items = country_subdivision_repo.find_by_country_id(country_id).await?;
        
        assert_eq!(found_items.len(), 3);
        let found_ids: Vec<Uuid> = found_items.iter().map(|item| item.id).collect();
        for saved_subdivision in &saved {
            assert!(found_ids.contains(&saved_subdivision.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_country_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        let non_existent_country_id = Uuid::new_v4();
        let found_items = country_subdivision_repo.find_by_country_id(non_existent_country_id).await?;
        
        assert!(found_items.is_empty());

        Ok(())
    }
}