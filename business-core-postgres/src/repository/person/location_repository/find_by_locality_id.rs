use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::location::LocationIdxModel;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub async fn find_by_locality_id(
        &self,
        locality_id: Uuid,
    ) -> Result<Vec<LocationIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.location_idx_cache.read().await;
        let items = cache.get_by_uuid_index("locality_id", &locality_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use crate::repository::person::test_utils::{
        create_test_audit_log, create_test_country, create_test_country_subdivision,
        create_test_locality, create_test_location,
    };

    #[tokio::test]
    async fn test_find_by_locality_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create country
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create country subdivision
        let country_subdivision = create_test_country_subdivision(country_id, "CA", "California");
        let country_subdivision_id = country_subdivision.id;
        country_subdivision_repo.create_batch(vec![country_subdivision], Some(audit_log.id)).await?;
        
        // Create locality
        let locality = create_test_locality(country_subdivision_id, "LA", "Los Angeles");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        // Create test locations belonging to the locality
        let mut locations = Vec::new();
        for i in 0..3 {
            let location = create_test_location(locality_id, &format!("location-{i}"));
            locations.push(location);
        }

        let saved = location_repo.create_batch(locations, Some(audit_log.id)).await?;

        // Find by locality_id
        let found = location_repo.find_by_locality_id(locality_id).await?;
        
        assert_eq!(found.len(), 3);
        for saved_location in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_location.id));
            assert!(found.iter().all(|idx| idx.locality_id == locality_id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_locality_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let location_repo = &ctx.person_repos().location_repository;

        let non_existent_locality_id = uuid::Uuid::new_v4();
        let found = location_repo.find_by_locality_id(non_existent_locality_id).await?;
        
        assert!(found.is_empty());

        Ok(())
    }
}