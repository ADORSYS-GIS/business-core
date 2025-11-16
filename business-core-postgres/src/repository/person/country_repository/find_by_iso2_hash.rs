use std::error::Error;

use business_core_db::models::person::country::CountryIdxModel;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub async fn find_by_iso2_hash(
        &self,
        iso2_hash: i64,
    ) -> Result<Vec<CountryIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.country_idx_cache.read().await;
        let items = cache.get_by_i64_index("iso2_hash", &iso2_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use heapless::String as HeaplessString;
    use super::super::test_utils::test_utils::create_test_country;

    #[tokio::test]
    async fn test_find_by_iso2_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut country_model = create_test_country("T3", "Test Country");
        let unique_iso2 = "T3";
        country_model.iso2 = HeaplessString::try_from(unique_iso2).unwrap();
        
        let saved = country_repo.create_batch(vec![country_model.clone()], None).await?;

        let unique_iso2_hash = hash_as_i64(&unique_iso2)?;
        let found_items = country_repo.find_by_iso2_hash(unique_iso2_hash).await?;
        
        assert_eq!(found_items.len(), 1);
        assert_eq!(found_items[0].id, saved[0].id);

        let non_existent_iso2 = "T4";
        let non_existent_iso2_hash = hash_as_i64(&non_existent_iso2)?;
        let found_items = country_repo.find_by_iso2_hash(non_existent_iso2_hash).await?;
        assert!(found_items.is_empty());

        Ok(())
    }
}