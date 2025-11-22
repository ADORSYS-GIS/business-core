use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;
use business_core_db::models::audit::audit_link::{AuditLinkModel, AuditEntityType};
use business_core_db::utils::hash_as_i64;
use business_core_db::repository::load_batch::LoadBatch;

#[async_trait]
impl DeleteBatch for AccountGlMappingRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Uuid,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        delete_batch_impl(self, ids, audit_log_id).await
    }
}

pub(super) async fn delete_batch_impl(
    repo: &AccountGlMappingRepositoryImpl,
    ids: &[Uuid],
    audit_log_id: Uuid,
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(0);
    }

    let entities_to_delete = repo.load_batch(ids).await?;
    let mut rows_affected = 0;

    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

    for entity_option in &entities_to_delete {
        if let Some(entity) = entity_option {
            let mut final_audit_entity = entity.clone();
            
            final_audit_entity.antecedent_hash = entity.hash;
            final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;
            
            final_audit_entity.audit_log_id = Some(audit_log_id);
            final_audit_entity.hash = 0;
            
            let final_hash = hash_as_i64(&final_audit_entity)?;
            final_audit_entity.hash = final_hash;
            
            sqlx::query(
                r#"
                INSERT INTO account_gl_mapping_audit (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(final_audit_entity.id)
            .bind(&final_audit_entity.customer_account_code)
            .bind(&final_audit_entity.overdraft_code)
            .bind(final_audit_entity.hash)
            .bind(final_audit_entity.audit_log_id)
            .bind(final_audit_entity.antecedent_hash)
            .bind(final_audit_entity.antecedent_audit_log_id)
            .execute(&mut **transaction)
            .await?;
            
            let result = sqlx::query(
                r#"
                DELETE FROM account_gl_mapping WHERE id = $1
                "#,
            )
            .bind(entity.id)
            .execute(&mut **transaction)
            .await?;
            
            rows_affected += result.rows_affected() as usize;
            
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
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
        }
    }
    
    drop(tx);
    
    {
        let idx_cache = repo.account_gl_mapping_idx_cache.read().await;
        let main_cache = repo.account_gl_mapping_cache.read().await;
        
        for id in ids {
            idx_cache.remove(id);
            main_cache.remove(id);
        }
    }
    
    Ok(rows_affected)
}
#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, create_test_audit_log};
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;

    #[tokio::test]
    async fn test_delete_batch_removes_from_caches_and_creates_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let account_gl_mapping_repo = &ctx.product_repos().account_gl_mapping_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![create_test_account_gl_mapping("12345")];
        let saved = account_gl_mapping_repo.create_batch(items, audit_log.id).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // Delete entities
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = account_gl_mapping_repo.delete_batch(&ids, delete_audit_log.id).await?;
        assert_eq!(deleted_count, ids.len());

        // Verify removed from both caches
        let idx_cache = account_gl_mapping_repo.account_gl_mapping_idx_cache.read().await;
        let main_cache = account_gl_mapping_repo.account_gl_mapping_cache.read().await;
        for id in &ids {
            assert!(!idx_cache.contains_primary(id), "Entity should be removed from index cache");
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }
}