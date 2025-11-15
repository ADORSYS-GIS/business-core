use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountrySubdivisionRepositoryImpl;

impl CountrySubdivisionRepositoryImpl {
    pub async fn find_ids_by_code_hash(
        &self,
        code_hash: i64,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.country_subdivision_idx_cache.read().await;
        let items = cache.get_by_i64_index("code_hash", &code_hash);
        let result = items.into_iter().map(|item| item.id).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use heapless::String as HeaplessString;
    use super::super::test_utils::test_utils::{create_test_country, create_test_country_subdivision};

    #[tokio::test]
    async fn test_find_ids_by_code_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("FR", "France");
        let country_id = country.id;
        country_repo.create_batch(vec![country], None).await?;

        let mut subdivision = create_test_country_subdivision(
            country_id,
            "TC1",
            "Test Code Subdivision",
        );
        let unique_code = "TC1";
        subdivision.code = HeaplessString::try_from(unique_code).unwrap();
        
        let saved = country_subdivision_repo.create_batch(vec![subdivision.clone()], None).await?;

        let unique_code_hash = hash_as_i64(&unique_code)?;
        let found_ids = country_subdivision_repo.find_ids_by_code_hash(unique_code_hash).await?;
        
        assert_eq!(found_ids.len(), 1);
        assert_eq!(found_ids[0], saved[0].id);

        Ok(())
    }

    #[tokio::test]
    async fn test_find_ids_by_code_hash_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;

        let non_existent_code = "NONEXIST";
        let non_existent_code_hash = hash_as_i64(&non_existent_code)?;
        let found_ids = country_subdivision_repo.find_ids_by_code_hash(non_existent_code_hash).await?;
        
        assert!(found_ids.is_empty());

        Ok(())
    }
}