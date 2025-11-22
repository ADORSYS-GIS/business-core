use async_trait::async_trait;
use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use business_core_db::repository::load_batch::LoadBatch;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;
use crate::utils::TryFromRow;

#[async_trait]
impl LoadBatch<AccountGlMappingModel> for AccountGlMappingRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<AccountGlMappingModel>>, Box<dyn Error + Send + Sync>> {
        load_batch_impl(self, ids).await
    }
}

pub(super) async fn load_batch_impl(
    repo: &AccountGlMappingRepositoryImpl,
    ids: &[Uuid],
) -> Result<Vec<Option<AccountGlMappingModel>>, Box<dyn Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    
    let main_cache = repo.account_gl_mapping_cache.read().await;
    let mut result = Vec::with_capacity(ids.len());
    let mut missing_ids = Vec::new();
    
    for &id in ids {
        match main_cache.get(&id) {
            Some(item) => result.push(Some(item)),
            None => {
                result.push(None);
                missing_ids.push(id);
            }
        }
    }
    
    drop(main_cache);
    
    if missing_ids.is_empty() {
        return Ok(result);
    }
    
    let query = r#"SELECT * FROM account_gl_mapping WHERE id = ANY($1)"#;
    let rows = {
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        sqlx::query(query).bind(&missing_ids).fetch_all(&mut **transaction).await?
    };
    
    let mut loaded_map = std::collections::HashMap::new();
    for row in rows {
        let item = AccountGlMappingModel::try_from_row(&row)?;
        loaded_map.insert(item.id, item);
    }
    
    let main_cache = repo.account_gl_mapping_cache.read().await;
    for (i, &id) in ids.iter().enumerate() {
        if result[i].is_none() {
            if let Some(item) = loaded_map.remove(&id) {
                main_cache.insert(item.clone());
                result[i] = Some(item);
            }
        }
    }
    
    Ok(result)
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, create_test_audit_log};
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let account_gl_mapping_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![create_test_account_gl_mapping("12345")];
        let saved = account_gl_mapping_repo.create_batch(items, audit_log.id).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // First load - should populate cache
        let loaded1 = account_gl_mapping_repo.load_batch(&ids).await?;
        
        // Second load - should hit cache
        let loaded2 = account_gl_mapping_repo.load_batch(&ids).await?;
        
        assert_eq!(loaded1.len(), loaded2.len());
        
        // Verify cache statistics
        let main_cache = account_gl_mapping_repo.account_gl_mapping_cache.read().await;
        let stats = main_cache.statistics();
        assert!(stats.hits() > 0, "Should have cache hits on second load");

        Ok(())
    }
}