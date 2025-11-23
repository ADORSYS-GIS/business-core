use async_trait::async_trait;
use business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &FeeTypeGlMappingRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<FeeTypeGlMappingModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM fee_type_gl_mapping WHERE id = ANY($1)"#;
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
            let item = FeeTypeGlMappingModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, FeeTypeGlMappingModel> for FeeTypeGlMappingRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<FeeTypeGlMappingModel>>, Box<dyn Error + Send + Sync>> {
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
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut fee_type_gl_mapping_entities = Vec::new();
        for i in 0..3 {
            let fee_type_gl_mapping = create_test_fee_type_gl_mapping(&format!("LOAD{i}"));
            fee_type_gl_mapping_entities.push(fee_type_gl_mapping);
        }

        let saved = fee_type_gl_mapping_repo.create_batch(fee_type_gl_mapping_entities.clone(), Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = fee_type_gl_mapping_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let fee_type_gl_mapping = item.unwrap();
            assert!(fee_type_gl_mapping.gl_code.as_str().starts_with("LOAD"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let fee_type_gl_mapping = create_test_fee_type_gl_mapping("SINGLE");

        let saved = fee_type_gl_mapping_repo.create_batch(vec![fee_type_gl_mapping], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = fee_type_gl_mapping_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}