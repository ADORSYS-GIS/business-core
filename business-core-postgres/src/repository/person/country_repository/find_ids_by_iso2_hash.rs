use std::error::Error;
use uuid::Uuid;

use super::repo_impl::CountryRepositoryImpl;

impl CountryRepositoryImpl {
    pub async fn find_ids_by_iso2_hash(
        &self,
        iso2_hash: i64,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.country_idx_cache.read();
        let result = cache
            .get_by_i64_index("iso2_hash", &iso2_hash)
            .map(|ids| ids.clone())
            .unwrap_or_default();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::repository::create_batch::CreateBatch;
    use crate::utils::hash_as_i64;
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
    async fn test_find_ids_by_iso2_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let country_repo = &ctx.person_repos().country_repository;

        let mut country_model = create_test_country("T3", "Test Country");
        let unique_iso2 = "T3";
        country_model.iso2 = HeaplessString::try_from(unique_iso2).unwrap();
        
        let audit_log_id = Uuid::new_v4();
        let saved = country_repo.create_batch(vec![country_model.clone()], audit_log_id).await?;

        let unique_iso2_hash = hash_as_i64(&unique_iso2);
        let found_ids = country_repo.find_ids_by_iso2_hash(unique_iso2_hash).await?;
        
        assert_eq!(found_ids.len(), 1);
        assert_eq!(found_ids[0], saved[0].id);

        let non_existent_iso2 = "T4";
        let non_existent_iso2_hash = hash_as_i64(&non_existent_iso2);
        let found_ids = country_repo.find_ids_by_iso2_hash(non_existent_iso2_hash).await?;
        assert!(found_ids.is_empty());

        Ok(())
    }
}