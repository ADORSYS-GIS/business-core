use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::location::LocationIdxModel;
use business_core_db::repository::pagination::{Page, PageRequest};

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub async fn find_by_locality_id(
        &self,
        locality_id: Uuid,
        page: PageRequest,
    ) -> Result<Page<LocationIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.location_idx_cache.read().await;
        let all_items = cache.get_by_uuid_index("locality_id", &locality_id);
        
        let total = all_items.len();
        let start = page.offset;
        let end = (start + page.limit).min(total);
        
        let items = if start < total {
            all_items[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        Ok(Page::new(items, total, page.limit, page.offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::pagination::PageRequest;
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
        let page = location_repo.find_by_locality_id(locality_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 3);
        for saved_location in &saved {
            assert!(page.items.iter().any(|idx| idx.id == saved_location.id));
            assert!(page.items.iter().all(|idx| idx.locality_id == locality_id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_locality_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let location_repo = &ctx.person_repos().location_repository;

        let non_existent_locality_id = uuid::Uuid::new_v4();
        let page = location_repo.find_by_locality_id(non_existent_locality_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_locality_id_pagination() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        // Create 5 test locations belonging to the locality
        let mut locations = Vec::new();
        for i in 0..5 {
            let location = create_test_location(locality_id, &format!("location-{i}"));
            locations.push(location);
        }
        location_repo.create_batch(locations, Some(audit_log.id)).await?;

        // Test first page (limit 2)
        let page_1 = location_repo.find_by_locality_id(locality_id, PageRequest::new(2, 0)).await?;
        assert_eq!(page_1.total, 5);
        assert_eq!(page_1.items.len(), 2);
        assert_eq!(page_1.page_number(), 1);
        assert_eq!(page_1.total_pages(), 3);
        assert!(page_1.has_more());

        // Test second page
        let page_2 = location_repo.find_by_locality_id(locality_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page_2.total, 5);
        assert_eq!(page_2.items.len(), 2);
        assert_eq!(page_2.page_number(), 2);
        assert!(page_2.has_more());

        // Test third page (only 1 item)
        let page_3 = location_repo.find_by_locality_id(locality_id, PageRequest::new(2, 4)).await?;
        assert_eq!(page_3.total, 5);
        assert_eq!(page_3.items.len(), 1);
        assert_eq!(page_3.page_number(), 3);
        assert!(!page_3.has_more());

        Ok(())
    }
}