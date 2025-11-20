use async_trait::async_trait;
use business_core_db::models::person::person::PersonModel;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::pagination::{Page, PageRequest};
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn load_audits_impl(
        repo: &PersonRepositoryImpl,
        id: Uuid,
        page: PageRequest,
    ) -> Result<Page<PersonModel>, Box<dyn Error + Send + Sync>> {
        // First, get the total count of audit records for this entity
        let count_query = r#"SELECT COUNT(*) as count FROM person_audit WHERE id = $1"#;
        let total: i64 = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query_scalar(count_query)
                    .bind(id)
                    .fetch_one(&mut **transaction)
                    .await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };

        // Then fetch the paginated audit records, ordered by audit_log_id (most recent first)
        let query = r#"
            SELECT * FROM person_audit 
            WHERE id = $1 
            ORDER BY audit_log_id DESC
            LIMIT $2 OFFSET $3
        "#;
        
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query)
                    .bind(id)
                    .bind(page.limit as i64)
                    .bind(page.offset as i64)
                    .fetch_all(&mut **transaction)
                    .await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let item = PersonModel::try_from_row(&row)?;
            items.push(item);
        }

        Ok(Page::new(items, total as usize, page.limit, page.offset))
    }
}

#[async_trait]
impl LoadAudits<Postgres, PersonModel> for PersonRepositoryImpl {
    async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<PersonModel>, Box<dyn Error + Send + Sync>> {
        Self::load_audits_impl(self, id, page).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_audits::LoadAudits;
    use business_core_db::repository::pagination::PageRequest;
    use business_core_db::repository::update_batch::UpdateBatch;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use heapless::String as HeaplessString;

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        // Create initial person
        let person = create_test_person("John Doe");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let mut saved = person_repo.create_batch(vec![person.clone()], Some(audit_log.id)).await?;

        // Update the person multiple times to create audit history
        // IMPORTANT: Must capture the returned updated entity to get the new hash and audit_log_id
        // This prevents "Concurrent update detected" errors on subsequent updates
        for i in 1..=3 {
            let audit_log = create_test_audit_log();
            audit_log_repo.create(&audit_log).await?;
            
            let mut updated = saved[0].clone();
            // Modify a field to create a new version
            updated.display_name = HeaplessString::try_from(format!("John Doe {i}").as_str()).unwrap();
            saved = person_repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        // Load first page of audit records
        let page = person_repo.load_audits(person_id, PageRequest::new(2, 0)).await?;

        // Should have 4 total audit records (1 create + 3 updates)
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = person_repo.load_audits(person_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2); // Second page with remaining 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;

        // Try to load audits for non-existing entity
        let non_existing_id = uuid::Uuid::new_v4();
        let page = person_repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}