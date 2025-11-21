use business_core_db::models::audit::AuditLinkModel;
use business_core_db::repository::pagination::{Page, PageRequest};
use uuid::Uuid;
use super::repo_impl::AuditLinkRepositoryImpl;

impl AuditLinkRepositoryImpl {
    pub(super) async fn find_by_audit_log_id_impl(
        repo: &AuditLinkRepositoryImpl,
        audit_log_id: Uuid,
        page: PageRequest,
    ) -> Result<Page<AuditLinkModel>, sqlx::Error> {
        // First, get the total count of audit links for this audit log
        let count_query = r#"SELECT COUNT(*) as count FROM audit_link WHERE audit_log_id = $1"#;
        let total: i64 = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query_scalar(count_query)
                    .bind(audit_log_id)
                    .fetch_one(&mut **transaction)
                    .await?
            } else {
                return Err(sqlx::Error::Configuration("Transaction has been consumed".into()));
            }
        };

        // Then fetch the paginated audit links
        let query = sqlx::query_as::<_, AuditLinkModel>(
            r#"
            SELECT audit_log_id, entity_id, entity_type
            FROM audit_link
            WHERE audit_log_id = $1
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(audit_log_id)
        .bind(page.limit as i64)
        .bind(page.offset as i64);

        let items = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.fetch_all(&mut **transaction).await?
            } else {
                return Err(sqlx::Error::Configuration("Transaction has been consumed".into()));
            }
        };

        Ok(Page::new(items, total as usize, page.limit, page.offset))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use crate::repository::person::test_utils::create_test_audit_log;
    use business_core_db::models::EntityType;
    use business_core_db::models::audit::AuditLinkModel;
    use business_core_db::repository::pagination::PageRequest;

    fn create_test_audit_link(audit_log_id: uuid::Uuid, entity_id: uuid::Uuid, entity_type: EntityType) -> AuditLinkModel {
        AuditLinkModel {
            audit_log_id,
            entity_id,
            entity_type: entity_type,
        }
    }

    #[tokio::test]
    async fn test_find_by_audit_log_id_paginated() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let audit_link_repo = &ctx.audit_repos().audit_link_repository;

        // Create an audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create multiple audit links for this audit log
        for _i in 0..5 {
            let entity_id = uuid::Uuid::new_v4();
            let audit_link = create_test_audit_link(audit_log.id, entity_id, EntityType::Location);
            audit_link_repo.create(&audit_link).await?;
        }

        // Load first page with limit of 2
        let page = audit_link_repo.find_by_audit_log_id(audit_log.id, PageRequest::new(2, 0)).await?;

        // Should have 5 total audit links
        assert_eq!(page.total, 5);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 3);
        assert!(page.has_more());

        // Load second page
        let page2 = audit_link_repo.find_by_audit_log_id(audit_log.id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 5);
        assert_eq!(page2.items.len(), 2); // Second page with 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(page2.has_more());

        // Load third page
        let page3 = audit_link_repo.find_by_audit_log_id(audit_log.id, PageRequest::new(2, 4)).await?;
        assert_eq!(page3.total, 5);
        assert_eq!(page3.items.len(), 1); // Last page with 1 record
        assert_eq!(page3.page_number(), 3);
        assert!(!page3.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_audit_log_id_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_link_repo = &ctx.audit_repos().audit_link_repository;

        // Try to find audit links for non-existing audit log
        let non_existing_id = uuid::Uuid::new_v4();
        let page = audit_link_repo.find_by_audit_log_id(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_audit_log_id_single_page() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let audit_link_repo = &ctx.audit_repos().audit_link_repository;

        // Create an audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create 3 audit links
        for _i in 0..3 {
            let entity_id = uuid::Uuid::new_v4();
            let audit_link = create_test_audit_link(audit_log.id, entity_id, EntityType::ComplianceStatus);
            audit_link_repo.create(&audit_link).await?;
        }

        // Load all with a large page size
        let page = audit_link_repo.find_by_audit_log_id(audit_log.id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 1);
        assert!(!page.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_audit_log_id_verify_audit_log_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let audit_link_repo = &ctx.audit_repos().audit_link_repository;

        // Create two different audit logs
        let audit_log1 = create_test_audit_log();
        let audit_log2 = create_test_audit_log();
        audit_log_repo.create(&audit_log1).await?;
        audit_log_repo.create(&audit_log2).await?;

        // Create 3 audit links for first audit log
        for _i in 0..3 {
            let entity_id = uuid::Uuid::new_v4();
            let audit_link = create_test_audit_link(audit_log1.id, entity_id, EntityType::Document);
            audit_link_repo.create(&audit_link).await?;
        }

        // Create 2 audit links for second audit log
        for _i in 0..2 {
            let entity_id = uuid::Uuid::new_v4();
            let audit_link = create_test_audit_link(audit_log2.id, entity_id, EntityType::EntityReference);
            audit_link_repo.create(&audit_link).await?;
        }

        // Verify first audit log has 3 links
        let page1 = audit_link_repo.find_by_audit_log_id(audit_log1.id, PageRequest::new(20, 0)).await?;
        assert_eq!(page1.total, 3);
        assert_eq!(page1.items.len(), 3);
        assert!(page1.items.iter().all(|link| link.audit_log_id == audit_log1.id));

        // Verify second audit log has 2 links
        let page2 = audit_link_repo.find_by_audit_log_id(audit_log2.id, PageRequest::new(20, 0)).await?;
        assert_eq!(page2.total, 2);
        assert_eq!(page2.items.len(), 2);
        assert!(page2.items.iter().all(|link| link.audit_log_id == audit_log2.id));

        Ok(())
    }
}