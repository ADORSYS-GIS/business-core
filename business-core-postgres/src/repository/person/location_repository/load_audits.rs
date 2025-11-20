use async_trait::async_trait;
use business_core_db::models::person::location::LocationModel;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::pagination::{Page, PageRequest};
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn load_audits_impl(
        repo: &LocationRepositoryImpl,
        id: Uuid,
        page: PageRequest,
    ) -> Result<Page<LocationModel>, Box<dyn Error + Send + Sync>> {
        // First, get the total count of audit records for this entity
        let count_query = r#"SELECT COUNT(*) as count FROM location_audit WHERE id = $1"#;
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
            SELECT * FROM location_audit 
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
            let item = LocationModel::try_from_row(&row)?;
            items.push(item);
        }

        Ok(Page::new(items, total as usize, page.limit, page.offset))
    }
}

#[async_trait]
impl LoadAudits<Postgres, LocationModel> for LocationRepositoryImpl {
    async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<LocationModel>, Box<dyn Error + Send + Sync>> {
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
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_country, create_test_country_subdivision, create_test_locality, create_test_location};
    use heapless::String as HeaplessString;

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("FR", "France");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "75", "Paris");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], Some(audit_log.id)).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "75001", "Paris 1er");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        // Create initial location with its own audit log
        let location = create_test_location(locality_id, "1 Rue de Rivoli");
        let location_id = location.id;
        let location_audit_log = create_test_audit_log();
        audit_log_repo.create(&location_audit_log).await?;
        let mut saved = location_repo.create_batch(vec![location.clone()], Some(location_audit_log.id)).await?;

        // Update the location multiple times to create audit history
        for i in 1..=3 {
            let audit_log = create_test_audit_log();
            audit_log_repo.create(&audit_log).await?;
            
            let mut updated = saved[0].clone();
            updated.street_line1 = HeaplessString::try_from(format!("{i} Rue de Rivoli").as_str()).unwrap();
            saved = location_repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        // Load first page of audit records
        let page = location_repo.load_audits(location_id, PageRequest::new(2, 0)).await?;

        // Should have 4 total audit records (1 create + 3 updates)
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = location_repo.load_audits(location_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 3);
        assert_eq!(page2.items.len(), 1); // Second page with remaining 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let location_repo = &ctx.person_repos().location_repository;

        // Try to load audits for non-existing location
        let non_existing_id = uuid::Uuid::new_v4();
        let page = location_repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}