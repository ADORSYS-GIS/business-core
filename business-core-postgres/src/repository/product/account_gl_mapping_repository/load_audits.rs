use async_trait::async_trait;
use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::page_request::PageRequest;
use business_core_db::repository::page::Page;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;
use crate::utils::TryFromRow;
use sqlx::Row;

#[async_trait]
impl LoadAudits<AccountGlMappingModel> for AccountGlMappingRepositoryImpl {
    async fn load_audits(
        &self,
        id: Uuid,
        page_request: PageRequest,
    ) -> Result<Page<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

        let count_query = "SELECT COUNT(*) FROM account_gl_mapping_audit WHERE id = $1";
        let total: i64 = sqlx::query(count_query)
            .bind(id)
            .fetch_one(&mut **transaction)
            .await?
            .get(0);

        let query = "SELECT * FROM account_gl_mapping_audit WHERE id = $1 ORDER BY audit_log_id DESC LIMIT $2 OFFSET $3";
        let rows = sqlx::query(query)
            .bind(id)
            .bind(page_request.size)
            .bind(page_request.offset)
            .fetch_all(&mut **transaction)
            .await?;

        let items = rows
            .iter()
            .map(AccountGlMappingModel::try_from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page::new(items, total, page_request.size, page_request.offset))
    }
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, create_test_audit_log};
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use business_core_db::repository::load_audits::LoadAudits;
    use business_core_db::repository::page_request::PageRequest;
    use uuid::Uuid;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use heapless::String;

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let account_gl_mapping_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Create initial entity
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let account_gl_mapping = create_test_account_gl_mapping("12345");
        let account_gl_mapping_id = account_gl_mapping.id;
        let mut saved = account_gl_mapping_repo.create_batch(vec![account_gl_mapping], audit_log.id).await?;

        // Update entity multiple times to create audit history
        for i in 1..=3 {
            let update_audit_log = create_test_audit_log();
            audit_log_repo.create(&update_audit_log).await?;
            
            let mut updated = saved[0].clone();
            updated.customer_account_code = String::from(format!("5432{}", i).as_str());
            saved = account_gl_mapping_repo.update_batch(vec![updated], update_audit_log.id).await?;
        }

        // Load audit history with pagination
        let page = account_gl_mapping_repo.load_audits(account_gl_mapping_id, PageRequest::new(2, 0)).await?;

        assert_eq!(page.total, 4); // 1 create + 3 updates
        assert_eq!(page.items.len(), 2);
        assert!(page.has_more());

        Ok(())
    }
}