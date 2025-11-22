use async_trait::async_trait;
use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use business_core_db::repository::create_batch::CreateBatch;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;
use business_core_db::models::audit::audit_link::{AuditLinkModel, AuditEntityType};

#[async_trait]
impl CreateBatch<AccountGlMappingModel> for AccountGlMappingRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<AccountGlMappingModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        create_batch_impl(self, items, audit_log_id).await
    }
}

pub(super) async fn create_batch_impl(
    repo: &AccountGlMappingRepositoryImpl,
    items: Vec<AccountGlMappingModel>,
    audit_log_id: Uuid,
) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let mut saved_items = Vec::new();
    let mut indices = Vec::new();
    
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    
    for mut item in items {
        let mut entity_for_hashing = item.clone();
        entity_for_hashing.hash = 0;
        entity_for_hashing.audit_log_id = Some(audit_log_id);
        
        let computed_hash = hash_as_i64(&entity_for_hashing)?;
        
        item.hash = computed_hash;
        item.audit_log_id = Some(audit_log_id);
        
        sqlx::query(
            r#"
            INSERT INTO account_gl_mapping_audit (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(item.id)
        .bind(&item.customer_account_code)
        .bind(&item.overdraft_code)
        .bind(item.hash)
        .bind(item.audit_log_id)
        .bind(item.antecedent_hash)
        .bind(item.antecedent_audit_log_id)
        .execute(&mut **transaction)
        .await?;
        
        sqlx::query(
            r#"
            INSERT INTO account_gl_mapping (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(item.id)
        .bind(&item.customer_account_code)
        .bind(&item.overdraft_code)
        .bind(item.hash)
        .bind(item.audit_log_id)
        .bind(item.antecedent_hash)
        .bind(item.antecedent_audit_log_id)
        .execute(&mut **transaction)
        .await?;
        
        let idx = item.to_index();
        sqlx::query(
            r#"
            INSERT INTO account_gl_mapping_idx (id)
            VALUES ($1)
            "#,
        )
        .bind(idx.id)
        .execute(&mut **transaction)
        .await?;
        
        let audit_link = AuditLinkModel {
            audit_log_id,
            entity_id: item.id,
            entity_type: AuditEntityType::AccountGlMapping,
        };
        sqlx::query(
            r#"
            INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_link.audit_log_id)
        .bind(audit_link.entity_id)
        .bind(audit_link.entity_type)
        .execute(&mut **transaction)
        .await?;
        
        indices.push(idx);
        saved_items.push(item);
    }
    
    drop(tx);
    
    {
        let idx_cache = repo.account_gl_mapping_idx_cache.read().await;
        let main_cache = repo.account_gl_mapping_cache.read().await;
        
        for (idx, item) in indices.iter().zip(saved_items.iter()) {
            idx_cache.add(idx.clone());
            main_cache.insert(item.clone());
        }
    }

    Ok(saved_items)
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, create_test_audit_log};
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;

    #[tokio::test]
    async fn test_create_batch_updates_caches_and_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let account_gl_mapping_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let items = vec![create_test_account_gl_mapping("12345")];
        let saved = account_gl_mapping_repo.create_batch(items, audit_log.id).await?;

        // Verify entities are in main cache
        let main_cache = account_gl_mapping_repo.account_gl_mapping_cache.read().await;
        for item in &saved {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
            assert!(item.hash != 0, "Entity should have computed hash");
            assert!(item.audit_log_id.is_some(), "Entity should have audit_log_id");
        }

        Ok(())
    }
}