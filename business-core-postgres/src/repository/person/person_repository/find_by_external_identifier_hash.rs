use std::error::Error;
use business_core_db::models::person::person::PersonIdxModel;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub async fn find_by_external_identifier_hash(
        &self,
        external_identifier_hash: i64,
    ) -> Result<Vec<PersonIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.person_idx_cache.read().await;
        let items = cache.get_by_i64_index("external_identifier_hash", &external_identifier_hash);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{random, setup_test_context};
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::utils::hash_as_i64;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};

    #[tokio::test]
    async fn test_find_by_external_identifier_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create test persons with same external identifier
        let external_id = format!("EMP-{}", random(5));
        let expected_hash = hash_as_i64(&external_id).unwrap();
        
        let mut persons = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("person-{i}"));
            person.external_identifier = Some(heapless::String::try_from(external_id.as_str()).unwrap());
            persons.push(person);
        }

        let saved = person_repo.create_batch(persons, Some(audit_log.id)).await?;

        // Find by external_identifier_hash
        let found = person_repo.find_by_external_identifier_hash(expected_hash).await?;
        
        assert_eq!(found.len(), 3);
        for saved_person in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_person.id));
            assert!(found.iter().all(|idx| idx.external_identifier_hash == Some(expected_hash)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_external_identifier_hash_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;

        let non_existent_hash = hash_as_i64(&"non-existent-id").unwrap();
        let found = person_repo.find_by_external_identifier_hash(non_existent_hash).await?;
        
        assert!(found.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_external_identifier_hash_multiple_identifiers() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let external_id_1 = format!("EMP-{}", random(5));
        let external_id_2 = format!("EMP-{}", random(5));
        let expected_hash_1 = hash_as_i64(&external_id_1).unwrap();
        let expected_hash_2 = hash_as_i64(&external_id_2).unwrap();

        // Create persons with identifier 1
        let mut persons_1 = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("person-id1-{i}"));
            person.external_identifier = Some(heapless::String::try_from(external_id_1.as_str()).unwrap());
            persons_1.push(person);
        }
        person_repo.create_batch(persons_1, Some(audit_log.id)).await?;

        // Create persons with identifier 2
        let mut persons_2 = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("person-id2-{i}"));
            person.external_identifier = Some(heapless::String::try_from(external_id_2.as_str()).unwrap());
            persons_2.push(person);
        }
        person_repo.create_batch(persons_2, Some(audit_log.id)).await?;
        
        // Find by hash 1 should only return 2 items
        let found_1 = person_repo.find_by_external_identifier_hash(expected_hash_1).await?;
        assert_eq!(found_1.len(), 2);
        assert!(found_1.iter().all(|idx| idx.external_identifier_hash == Some(expected_hash_1)));

        // Find by hash 2 should only return 3 items
        let found_2 = person_repo.find_by_external_identifier_hash(expected_hash_2).await?;
        assert_eq!(found_2.len(), 3);
        assert!(found_2.iter().all(|idx| idx.external_identifier_hash == Some(expected_hash_2)));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_external_identifier_hash_with_none_values() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create persons without external identifier (None)
        let mut persons_without_id = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("person-no-id-{i}"));
            person.external_identifier = None;
            persons_without_id.push(person);
        }
        person_repo.create_batch(persons_without_id, Some(audit_log.id)).await?;

        // Create persons with external identifier
        let external_id = format!("EMP-{}", random(5));
        let expected_hash = hash_as_i64(&external_id).unwrap();
        
        let mut persons_with_id = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("person-with-id-{i}"));
            person.external_identifier = Some(heapless::String::try_from(external_id.as_str()).unwrap());
            persons_with_id.push(person);
        }
        person_repo.create_batch(persons_with_id, Some(audit_log.id)).await?;

        // Find by hash should only return persons with that specific identifier
        let found = person_repo.find_by_external_identifier_hash(expected_hash).await?;
        assert_eq!(found.len(), 3);
        assert!(found.iter().all(|idx| idx.external_identifier_hash == Some(expected_hash)));

        Ok(())
    }
}