use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::entity_reference::EntityReferenceIdxModel;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub async fn find_by_person_id(
        &self,
        person_id: Uuid,
    ) -> Result<Vec<EntityReferenceIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.entity_reference_idx_cache.read().await;
        let items = cache.get_by_uuid_index("person_id", &person_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
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
                &format!("external-ref-{}", i),
            );
            entity_references.push(entity_ref);
        }

        let saved = entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        let found = entity_reference_repo.find_by_person_id(person_id).await?;
        
        assert_eq!(found.len(), 3);
        for saved_entity_ref in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_entity_ref.id));
            assert!(found.iter().all(|idx| idx.person_id == person_id));
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
        
        let found = entity_reference_repo.find_by_person_id(person_id).await?;
        
        assert!(found.is_empty());

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
                &format!("person1-ref-{}", i),
            ));
        }
        entity_reference_repo.create_batch(entity_refs_1, Some(audit_log.id)).await?;

        // Create entity references for person 2
        let mut entity_refs_2 = Vec::new();
        for i in 0..3 {
            entity_refs_2.push(create_test_entity_reference(
                person_id_2,
                &format!("person2-ref-{}", i),
            ));
        }
        entity_reference_repo.create_batch(entity_refs_2, Some(audit_log.id)).await?;

        // Find by person_id_1 should only return 2 items
        let found_1 = entity_reference_repo.find_by_person_id(person_id_1).await?;
        assert_eq!(found_1.len(), 2);
        assert!(found_1.iter().all(|idx| idx.person_id == person_id_1));

        // Find by person_id_2 should only return 3 items
        let found_2 = entity_reference_repo.find_by_person_id(person_id_2).await?;
        assert_eq!(found_2.len(), 3);
        assert!(found_2.iter().all(|idx| idx.person_id == person_id_2));

        Ok(())
    }
}