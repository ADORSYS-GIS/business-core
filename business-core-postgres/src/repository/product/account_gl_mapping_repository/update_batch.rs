use async_trait::async_trait;
use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use business_core_db::repository::update_batch::UpdateBatch;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;
use business_core_db::models::audit::audit_link::{AuditLinkModel, AuditEntityType};

#[async_trait]
impl UpdateBatch<AccountGlMappingModel> for AccountGlMappingRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<AccountGlMappingModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        update_batch_impl(self, items, audit_log_id).await
    }
}

pub(super) async fn update_batch_impl(
    repo: &AccountGlMappingRepositoryImpl,
    items: Vec<AccountGlMappingModel>,
    audit_log_id: Uuid,
) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let mut updated_items = Vec::new();
    let mut indices = Vec::new();
    
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    
    for mut item in items {
        let previous_hash = item.hash;
        let previous_audit_log_id = item.audit_log_id.ok_or("Entity must have audit_log_id for update")?;
        
        let mut entity_for_hashing = item.clone();
        entity_for_hashing.hash = 0;
        
        let computed_hash = hash_as_i64(&entity_for_hashing)?;
        
        if computed_hash == previous_hash {
            updated_items.push(item);
            continue;
        }
        
        item.antecedent_hash = previous_hash;
        item.antecedent_audit_log_id = previous_audit_log_id;
        
        item.audit_log_id = Some(audit_log_id);
        item.hash = 0;
        
        let new_computed_hash = hash_as_i64(&item)?;
        item.hash = new_computed_hash;
        
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
            UPDATE account_gl_mapping SET
                customer_account_code = $2,
                overdraft_code = $3,
                hash = $4,
                audit_log_id = $5,
                antecedent_hash = $6,
                antecedent_audit_log_id = $7
            WHERE id = $1 AND hash = $6 AND audit_log_id = $7
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
        
        indices.push((item.id, item.to_index()));
        updated_items.push(item);
    }
    
    drop(tx);
    
    {
        let idx_cache = repo.account_gl_mapping_idx_cache.read().await;
        let main_cache = repo.account_gl_mapping_cache.read().await;
        
        for (id, idx) in indices.iter() {
            idx_cache.remove(id);
            idx_cache.add(idx.clone());
            main_cache.update(updated_items.iter().find(|i| i.id == *id).unwrap().clone());
        }
    }

    Ok(updated_items)
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, create_test_audit_log};
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use uuid::Uuid;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use heapless::String;

    #[tokio::test]
    async fn test_update_batch_updates_caches_and_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let account_gl_mapping_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![create_test_account_gl_mapping("12345")];
        let mut saved = account_gl_mapping_repo.create_batch(items, audit_log.id).await?;
        
        // Store original hash
        let original_hash = saved[0].hash;
        
        // Update entity
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        
        saved[0].customer_account_code = String::from("54321");
        let updated = account_gl_mapping_repo.update_batch(saved, update_audit_log.id).await?;

        // Verify updated entity in cache with new hash
        let main_cache = account_gl_mapping_repo.account_gl_mapping_cache.read().await;
        let cached = main_cache.get(&updated[0].id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().customer_account_code, updated[0].customer_account_code);
        assert_ne!(cached.unwrap().hash, original_hash, "Hash should change after update");
        assert_eq!(cached.unwrap().antecedent_hash, original_hash, "Antecedent hash should match original");

        Ok(())
    }
}