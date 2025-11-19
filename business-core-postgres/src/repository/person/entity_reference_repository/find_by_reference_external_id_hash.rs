use std::error::Error;
use business_core_db::models::person::entity_reference::EntityReferenceIdxModel;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub async fn find_by_reference_external_id_hash(
        &self,
        reference_external_id_hash: i64,
    ) -> Result<Vec<EntityReferenceIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.entity_reference_idx_cache.read().await;
        let items = cache.get_by_i64_index("reference_external_id_hash", &reference_external_id_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person, create_test_entity_reference};

    #[tokio::test]
    async fn test_find_by_reference_external_id_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let external_id = "shared-external-ref";
        let expected_hash = hash_as_i64(&external_id).unwrap();
        
        let mut entity_references = Vec::new();
        for _i in 0..3 {
            let entity_ref = create_test_entity_reference(
                person_id,
                external_id,
            );
            entity_references.push(entity_ref);
        }

        let saved = entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        let found = entity_reference_repo.find_by_reference_external_id_hash(expected_hash).await?;
        
        assert_eq!(found.len(), 3);
        for saved_entity_ref in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_entity_ref.id));
            assert!(found.iter().all(|idx| idx.reference_external_id_hash == expected_hash));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_reference_external_id_hash_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        // Create a hash that doesn't exist in the database
        let non_existent_hash = hash_as_i64(&"non-existent-ref").unwrap();
        let found = entity_reference_repo.find_by_reference_external_id_hash(non_existent_hash).await?;
        
        assert!(found.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_reference_external_id_hash_multiple_hashes() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let external_id_1 = "external-ref-1";
        let external_id_2 = "external-ref-2";
        let expected_hash_1 = hash_as_i64(&external_id_1).unwrap();
        let expected_hash_2 = hash_as_i64(&external_id_2).unwrap();
        
        // Create entity references with hash 1
        let mut entity_refs_1 = Vec::new();
        for _ in 0..2 {
            entity_refs_1.push(create_test_entity_reference(
                person_id,
                external_id_1,
            ));
        }
        entity_reference_repo.create_batch(entity_refs_1, Some(audit_log.id)).await?;

        // Create entity references with hash 2
        let mut entity_refs_2 = Vec::new();
        for _ in 0..3 {
            entity_refs_2.push(create_test_entity_reference(
                person_id,
                external_id_2,
            ));
        }
        entity_reference_repo.create_batch(entity_refs_2, Some(audit_log.id)).await?;

        // Find by hash 1 should only return 2 items
        let found_1 = entity_reference_repo.find_by_reference_external_id_hash(expected_hash_1).await?;
        assert_eq!(found_1.len(), 2);
        assert!(found_1.iter().all(|idx| idx.reference_external_id_hash == expected_hash_1));

        // Find by hash 2 should only return 3 items
        let found_2 = entity_reference_repo.find_by_reference_external_id_hash(expected_hash_2).await?;
        assert_eq!(found_2.len(), 3);
        assert!(found_2.iter().all(|idx| idx.reference_external_id_hash == expected_hash_2));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_reference_external_id_hash_different_persons() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        let external_id = "shared-external-ref";
        let expected_hash = hash_as_i64(&external_id).unwrap();
        
        // Create entity references with same hash but different persons
        let entity_refs = vec![
            create_test_entity_reference(person_id_1, external_id),
            create_test_entity_reference(person_id_2, external_id),
        ];
        
        entity_reference_repo.create_batch(entity_refs, Some(audit_log.id)).await?;

        // Find by hash should return both items
        let found = entity_reference_repo.find_by_reference_external_id_hash(expected_hash).await?;
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|idx| idx.reference_external_id_hash == expected_hash));
        
        // Verify that we have references from both persons
        let person_ids: Vec<_> = found.iter().map(|idx| idx.person_id).collect();
        assert!(person_ids.contains(&person_id_1));
        assert!(person_ids.contains(&person_id_2));

        Ok(())
    }
}