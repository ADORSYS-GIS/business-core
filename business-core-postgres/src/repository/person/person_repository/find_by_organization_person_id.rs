use std::error::Error;
use uuid::Uuid;
use business_core_db::models::person::person::PersonIdxModel;
use business_core_db::repository::pagination::{Page, PageRequest};

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub async fn find_by_organization_person_id(
        &self,
        organization_person_id: Uuid,
        page: PageRequest,
    ) -> Result<Page<PersonIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.person_idx_cache.read().await;
        let all_items = cache.get_by_uuid_index("organization_person_id", &organization_person_id);
        let total = all_items.len();
        
        // Apply pagination
        let items: Vec<PersonIdxModel> = all_items
            .into_iter()
            .skip(page.offset)
            .take(page.limit)
            .collect();
        
        Ok(Page::new(items, total, page.limit, page.offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::pagination::PageRequest;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};

    #[tokio::test]
    async fn test_find_by_organization_person_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create organization person
        let org_person = create_test_person("organization-person");
        let org_person_id = org_person.id;
        person_repo.create_batch(vec![org_person], Some(audit_log.id)).await?;
        
        // Create test persons belonging to the organization
        let mut persons = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("employee-{i}"));
            person.organization_person_id = Some(org_person_id);
            persons.push(person);
        }

        let saved = person_repo.create_batch(persons, Some(audit_log.id)).await?;

        // Find by organization_person_id with pagination
        let page = person_repo.find_by_organization_person_id(org_person_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 3);
        for saved_person in &saved {
            assert!(page.items.iter().any(|idx| idx.id == saved_person.id));
            assert!(page.items.iter().all(|idx| idx.organization_person_id == Some(org_person_id)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_organization_person_id_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;

        let non_existent_org_id = uuid::Uuid::new_v4();
        let page = person_repo.find_by_organization_person_id(non_existent_org_id, PageRequest::new(10, 0)).await?;
        
        assert_eq!(page.total, 0);
        assert!(page.items.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_organization_person_id_multiple_organizations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create two organization persons
        let org_person_1 = create_test_person("organization-1");
        let org_person_id_1 = org_person_1.id;
        let org_person_2 = create_test_person("organization-2");
        let org_person_id_2 = org_person_2.id;
        
        person_repo.create_batch(vec![org_person_1, org_person_2], Some(audit_log.id)).await?;
        
        // Create employees for organization 1
        let mut employees_1 = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("org1-employee-{i}"));
            person.organization_person_id = Some(org_person_id_1);
            employees_1.push(person);
        }
        person_repo.create_batch(employees_1, Some(audit_log.id)).await?;

        // Create employees for organization 2
        let mut employees_2 = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("org2-employee-{i}"));
            person.organization_person_id = Some(org_person_id_2);
            employees_2.push(person);
        }
        person_repo.create_batch(employees_2, Some(audit_log.id)).await?;

        // Find by org_person_id_1 should only return 2 items
        let page_1 = person_repo.find_by_organization_person_id(org_person_id_1, PageRequest::new(10, 0)).await?;
        assert_eq!(page_1.total, 2);
        assert_eq!(page_1.items.len(), 2);
        assert!(page_1.items.iter().all(|idx| idx.organization_person_id == Some(org_person_id_1)));

        // Find by org_person_id_2 should only return 3 items
        let page_2 = person_repo.find_by_organization_person_id(org_person_id_2, PageRequest::new(10, 0)).await?;
        assert_eq!(page_2.total, 3);
        assert_eq!(page_2.items.len(), 3);
        assert!(page_2.items.iter().all(|idx| idx.organization_person_id == Some(org_person_id_2)));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_organization_person_id_with_none_values() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create organization person
        let org_person = create_test_person("organization-person");
        let org_person_id = org_person.id;
        person_repo.create_batch(vec![org_person], Some(audit_log.id)).await?;

        // Create persons without organization (None)
        let mut persons_without_org = Vec::new();
        for i in 0..2 {
            let mut person = create_test_person(&format!("independent-person-{i}"));
            person.organization_person_id = None;
            persons_without_org.push(person);
        }
        person_repo.create_batch(persons_without_org, Some(audit_log.id)).await?;

        // Create persons with organization
        let mut persons_with_org = Vec::new();
        for i in 0..3 {
            let mut person = create_test_person(&format!("org-employee-{i}"));
            person.organization_person_id = Some(org_person_id);
            persons_with_org.push(person);
        }
        person_repo.create_batch(persons_with_org, Some(audit_log.id)).await?;

        // Find by org_person_id should only return persons with that specific organization
        let page = person_repo.find_by_organization_person_id(org_person_id, PageRequest::new(10, 0)).await?;
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 3);
        assert!(page.items.iter().all(|idx| idx.organization_person_id == Some(org_person_id)));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_organization_person_id_pagination() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create audit log first
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create organization person
        let org_person = create_test_person("organization-person");
        let org_person_id = org_person.id;
        person_repo.create_batch(vec![org_person], Some(audit_log.id)).await?;
        
        // Create 5 test persons belonging to the organization
        let mut persons = Vec::new();
        for i in 0..5 {
            let mut person = create_test_person(&format!("employee-{i}"));
            person.organization_person_id = Some(org_person_id);
            persons.push(person);
        }

        person_repo.create_batch(persons, Some(audit_log.id)).await?;

        // Test first page (limit 2, offset 0)
        let page1 = person_repo.find_by_organization_person_id(org_person_id, PageRequest::new(2, 0)).await?;
        assert_eq!(page1.total, 5);
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.page_number(), 1);
        assert_eq!(page1.total_pages(), 3);
        assert!(page1.has_more());

        // Test second page (limit 2, offset 2)
        let page2 = person_repo.find_by_organization_person_id(org_person_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 5);
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.page_number(), 2);
        assert!(page2.has_more());

        // Test third page (limit 2, offset 4) - should have 1 item
        let page3 = person_repo.find_by_organization_person_id(org_person_id, PageRequest::new(2, 4)).await?;
        assert_eq!(page3.total, 5);
        assert_eq!(page3.items.len(), 1);
        assert_eq!(page3.page_number(), 3);
        assert!(!page3.has_more());

        Ok(())
    }
}