use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::entity_reference::EntityReferenceIdxModel;
use business_core_db::repository::pagination::{Page, PageRequest};

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub async fn find_by_person_id(
        &self,
        person_id: Uuid,
        page: PageRequest,
    ) -> Result<Page<EntityReferenceIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.entity_reference_idx_cache.read().await;
        let all_items = cache.get_by_uuid_index("person_id", &person_id);
        
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
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person, create_test_entity_reference};

    #[tokio::test]
    async fn test_find_by_person_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create test person
        let person = create_test_person("test-person");
        let person_id = person.id;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;
        
        let mut entity_references = Vec::new();
        for i in 0..3 {
            let entity_ref = create_test_entity_reference(
                person_id,
                &format!("external-ref-{i}"),
            );
            entity_references.push(entity_ref);
        }

        let saved = entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        let page = entity_reference_repo.find_by_person_id(person_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 3);
        for saved_entity_ref in &saved {
            assert!(page.items.iter().any(|idx| idx.id == saved_entity_ref.id));
            assert!(page.items.iter().all(|idx| idx.person_id == person_id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_person_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Create a person but don't create any entity references for it
        let person = create_test_person("test-person");
        let person_id = person.id;
        
        let page = entity_reference_repo.find_by_person_id(person_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_person_id_multiple_persons() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create two persons
        let person_1 = create_test_person("test-person-1");
        let person_id_1 = person_1.id;
        let person_2 = create_test_person("test-person-2");
        let person_id_2 = person_2.id;
        
        person_repo.create_batch(vec![person_1, person_2], Some(audit_log.id)).await?;
        
        // Create entity references for person 1
        let mut entity_refs_1 = Vec::new();
        for i in 0..2 {
            entity_refs_1.push(create_test_entity_reference(
                person_id_1,
                &format!("person1-ref-{i}"),
            ));
        }
        entity_reference_repo.create_batch(entity_refs_1, Some(audit_log.id)).await?;

        // Create entity references for person 2
        let mut entity_refs_2 = Vec::new();
        for i in 0..3 {
            entity_refs_2.push(create_test_entity_reference(
                person_id_2,
                &format!("person2-ref-{i}"),
            ));
        }
        entity_reference_repo.create_batch(entity_refs_2, Some(audit_log.id)).await?;

        // Find by person_id_1 should only return 2 items
        let page_1 = entity_reference_repo.find_by_person_id(person_id_1, PageRequest::new(10, 0)).await?;
        assert_eq!(page_1.total, 2);
        assert_eq!(page_1.items.len(), 2);
        assert!(page_1.items.iter().all(|idx| idx.person_id == person_id_1));

        // Find by person_id_2 should only return 3 items
        let page_2 = entity_reference_repo.find_by_person_id(person_id_2, PageRequest::new(10, 0)).await?;
        assert_eq!(page_2.total, 3);
        assert_eq!(page_2.items.len(), 3);
        assert!(page_2.items.iter().all(|idx| idx.person_id == person_id_2));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_person_id_pagination() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create test person
        let person = create_test_person("test-person");
        let person_id = person.id;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;
        
        // Create 5 entity references
        let mut entity_references = Vec::new();
        for i in 0..5 {
            let entity_ref = create_test_entity_reference(
                person_id,
                &format!("external-ref-{i}"),
            );
            entity_references.push(entity_ref);
        }
        entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        // Test first page (limit 2)
        let page_1 = entity_reference_repo.find_by_person_id(person_id, PageRequest::new(2, 0)).await?;
        assert_eq!(page_1.total, 5);
        assert_eq!(page_1.items.len(), 2);
        assert_eq!(page_1.page_number(), 1);
        assert_eq!(page_1.total_pages(), 3);
        assert!(page_1.has_more());

        // Test second page
        let page_2 = entity_reference_repo.find_by_person_id(person_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page_2.total, 5);
        assert_eq!(page_2.items.len(), 2);
        assert_eq!(page_2.page_number(), 2);
        assert!(page_2.has_more());

        // Test third page (only 1 item)
        let page_3 = entity_reference_repo.find_by_person_id(person_id, PageRequest::new(2, 4)).await?;
        assert_eq!(page_3.total, 5);
        assert_eq!(page_3.items.len(), 1);
        assert_eq!(page_3.page_number(), 3);
        assert!(!page_3.has_more());

        Ok(())
    }
}