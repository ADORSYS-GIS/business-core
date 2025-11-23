use async_trait::async_trait;
use business_core_db::models::product::product::ProductModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ProductRepositoryImpl;

impl ProductRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ProductRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ProductModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM product WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = ProductModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, ProductModel> for ProductRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<ProductModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use crate::repository::product::product_repository::test_utils::create_test_product;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;
    use business_core_db::models::description::named::NamedModel;
    use business_core_db::models::description::named_entity_type::NamedEntityType;
    use heapless::String as HeaplessString;

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
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let _ = named_repo.create_batch(vec![named], Some(audit_log.id)).await?;

        let account_gl = create_test_account_gl_mapping("ACC001");
        let fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        let saved_account_gl = account_gl_repo.create_batch(vec![account_gl], Some(audit_log.id)).await?;
        let saved_fee_type_gl = fee_type_gl_repo.create_batch(vec![fee_type_gl], Some(audit_log.id)).await?;

        let mut products = Vec::new();
        for i in 0..3 {
            let named = create_test_named(&format!("Product {i}"));
            let saved_named = named_repo.create_batch(vec![named], Some(audit_log.id)).await?;
            let product = create_test_product(
                saved_named[0].id,
                saved_account_gl[0].id,
                saved_fee_type_gl[0].id,
            );
            products.push(product);
        }

        let saved = product_repo.create_batch(products, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = product_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let product = item.unwrap();
            assert!(product.is_active);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named = create_test_named("Single Product");
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

        let saved = product_repo.create_batch(vec![product], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = product_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}