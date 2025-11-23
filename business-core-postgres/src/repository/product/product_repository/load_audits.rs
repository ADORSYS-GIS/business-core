use async_trait::async_trait;
use business_core_db::models::product::product::ProductModel;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::pagination::{Page, PageRequest};
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ProductRepositoryImpl;

impl ProductRepositoryImpl {
    pub(super) async fn load_audits_impl(
        repo: &ProductRepositoryImpl,
        id: Uuid,
        page: PageRequest,
    ) -> Result<Page<ProductModel>, Box<dyn Error + Send + Sync>> {
        let count_query = r#"SELECT COUNT(*) as count FROM product_audit WHERE id = $1"#;
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

        let query = r#"
            SELECT * FROM product_audit 
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
            let item = ProductModel::try_from_row(&row)?;
            items.push(item);
        }

        Ok(Page::new(items, total as usize, page.limit, page.offset))
    }
}

#[async_trait]
impl LoadAudits<Postgres, ProductModel> for ProductRepositoryImpl {
    async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<ProductModel>, Box<dyn Error + Send + Sync>> {
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
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::repository::product::product_repository::test_utils::create_test_product;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;
    use business_core_db::models::description::named::NamedModel;
    use business_core_db::models::description::named_entity_type::NamedEntityType;
    use heapless::String as HeaplessString;
    use uuid::Uuid;
    use rust_decimal::Decimal;

    fn create_test_named(name: &str) -> NamedModel {
        NamedModel {
            id: Uuid::new_v4(),
            entity_type: NamedEntityType::Country,
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
            name_l4: None,
            description_l1: None,
            description_l2: None,
            description_l3: None,
            description_l4: None,
            antecedent_hash: 0,
            antecedent_audit_log_id: Uuid::nil(),
            hash: 0,
            audit_log_id: None,
        }
    }

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named = create_test_named("Test Product");
        let saved_named = named_repo.create_batch(vec![named], Some(audit_log.id)).await?;

        let account_gl = create_test_account_gl_mapping("ACC001");
        let fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        let saved_account_gl = account_gl_repo.create_batch(vec![account_gl], Some(audit_log.id)).await?;
        let saved_fee_type_gl = fee_type_gl_repo.create_batch(vec![fee_type_gl], Some(audit_log.id)).await?;

        let product = create_test_product(
            saved_named[0].id,
            saved_account_gl[0].id,
            saved_fee_type_gl[0].id,
        );
        let product_id = product.id;
        let mut saved = product_repo.create_batch(vec![product.clone()], Some(audit_log.id)).await?;

        // Update the product multiple times to create audit history
        for i in 1..=3 {
            let audit_log = create_test_audit_log();
            audit_log_repo.create(&audit_log).await?;
            
            let mut updated = saved[0].clone();
            updated.minimum_balance = Decimal::from(100 + i * 100);
            saved = product_repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        // Load first page of audit records
        let page = product_repo.load_audits(product_id, PageRequest::new(2, 0)).await?;

        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = product_repo.load_audits(product_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let product_repo = &ctx.product_repos().product_repository;

        let non_existing_id = uuid::Uuid::new_v4();
        let page = product_repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}