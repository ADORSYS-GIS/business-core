use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::person::PersonIdxModel;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub async fn find_by_duplicate_of_person_id(
        &self,
        duplicate_of_person_id: Uuid,
    ) -> Result<Vec<PersonIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.person_idx_cache.read().await;
        let items = cache.get_by_uuid_index("duplicate_of_person_id", &duplicate_of_person_id);
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};

    #[tokio::test]
    async fn test_find_by_duplicate_of_person_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create original person
        let original_person = create_test_person("original-person");
        let original_person_id = original_person.id;
        person_repo.create_batch(vec![original_person], Some(audit_log.id)).await?;
        
        // Create duplicate persons that reference the original
        let mut duplicate_persons = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("duplicate-{}", i));
            person.duplicate_of_person_id = Some(original_person_id);
            duplicate_persons.push(person);
        }

        let saved = person_repo.create_batch(duplicate_persons, Some(audit_log.id)).await?;

        // Find by duplicate_of_person_id
        let found = person_repo.find_by_duplicate_of_person_id(original_person_id).await?;
        
        assert_eq!(found.len(), 3);
        for saved_person in &saved {
            assert!(found.iter().any(|idx| idx.id == saved_person.id));
            assert!(found.iter().all(|idx| idx.duplicate_of_person_id == Some(original_person_id)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_duplicate_of_person_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;

        let non_existent_person_id = uuid::Uuid::new_v4();
        let found = person_repo.find_by_duplicate_of_person_id(non_existent_person_id).await?;
        
        assert!(found.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_duplicate_of_person_id_multiple_originals() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create two original persons
        let original_person_1 = create_test_person("original-person-1");
        let original_person_id_1 = original_person_1.id;
        let original_person_2 = create_test_person("original-person-2");
        let original_person_id_2 = original_person_2.id;
        
        person_repo.create_batch(vec![original_person_1, original_person_2], Some(audit_log.id)).await?;
        
        // Create duplicates for original person 1
        let mut duplicates_1 = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("duplicate-of-1-{}", i));
            person.duplicate_of_person_id = Some(original_person_id_1);
            duplicates_1.push(person);
        }
        person_repo.create_batch(duplicates_1, Some(audit_log.id)).await?;

        // Create duplicates for original person 2
        let mut duplicates_2 = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("duplicate-of-2-{}", i));
            person.duplicate_of_person_id = Some(original_person_id_2);
            duplicates_2.push(person);
        }
        person_repo.create_batch(duplicates_2, Some(audit_log.id)).await?;

        // Find by original_person_id_1 should only return 2 items
        let found_1 = person_repo.find_by_duplicate_of_person_id(original_person_id_1).await?;
        assert_eq!(found_1.len(), 2);
        assert!(found_1.iter().all(|idx| idx.duplicate_of_person_id == Some(original_person_id_1)));

        // Find by original_person_id_2 should only return 3 items
        let found_2 = person_repo.find_by_duplicate_of_person_id(original_person_id_2).await?;
        assert_eq!(found_2.len(), 3);
        assert!(found_2.iter().all(|idx| idx.duplicate_of_person_id == Some(original_person_id_2)));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_duplicate_of_person_id_with_none_values() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create original person
        let original_person = create_test_person("original-person");
        let original_person_id = original_person.id;
        person_repo.create_batch(vec![original_person], Some(audit_log.id)).await?;

        // Create persons that are not duplicates (None)
        let mut non_duplicate_persons = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("unique-person-{}", i));
            person.duplicate_of_person_id = None;
            non_duplicate_persons.push(person);
        }
        person_repo.create_batch(non_duplicate_persons, Some(audit_log.id)).await?;

        // Create duplicate persons
        let mut duplicate_persons = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("duplicate-person-{}", i));
            person.duplicate_of_person_id = Some(original_person_id);
            duplicate_persons.push(person);
        }
        person_repo.create_batch(duplicate_persons, Some(audit_log.id)).await?;

        // Find by original_person_id should only return duplicate persons
        let found = person_repo.find_by_duplicate_of_person_id(original_person_id).await?;
        assert_eq!(found.len(), 3);
        assert!(found.iter().all(|idx| idx.duplicate_of_person_id == Some(original_person_id)));

        Ok(())
    }
}