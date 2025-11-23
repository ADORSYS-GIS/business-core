use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    product::fee_type_gl_mapping::FeeTypeGlMappingModel,
};
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for FeeTypeGlMappingModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices_to_update = Vec::new();
        
        {
            let mut tx = self.executor.tx.lock().await;
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
                    INSERT INTO fee_type_gl_mapping_audit
                    (id, fee_type, gl_code, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7)
                    "#,
                )
                .bind(item.id)
                .bind(item.fee_type.clone())
                .bind(item.gl_code.as_str())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE fee_type_gl_mapping SET
                    fee_type = $2,
                    gl_code = $3,
                    antecedent_hash = $4,
                    antecedent_audit_log_id = $5,
                    hash = $6,
                    audit_log_id = $7
                    WHERE id = $1 AND hash = $8 AND audit_log_id = $9
                    "#,
                )
                .bind(item.id)
                .bind(item.fee_type.clone())
                .bind(item.gl_code.as_str())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .bind(previous_hash)
                .bind(previous_audit_log_id)
                .execute(&mut **transaction)
                .await?
                .rows_affected();

                if rows_affected == 0 {
                    return Err("Concurrent update detected".into());
                }

                let idx = item.to_index();
                sqlx::query(
                    r#"
                    UPDATE fee_type_gl_mapping_idx SET
                    fee_type = $2,
                    gl_code = $3
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.fee_type.clone())
                .bind(idx.gl_code.as_str())
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::FeeTypeGlMapping,
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

                indices_to_update.push((item.id, idx));
                updated_items.push(item);
            }
        }
        
        {
            let cache = self.fee_type_gl_mapping_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, FeeTypeGlMappingModel> for FeeTypeGlMappingRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut fee_type_gl_mapping_entities = Vec::new();
        for i in 0..3 {
            let fee_type_gl_mapping = create_test_fee_type_gl_mapping(&format!("ORIG{i}"));
            fee_type_gl_mapping_entities.push(fee_type_gl_mapping);
        }

        let saved = fee_type_gl_mapping_repo.create_batch(fee_type_gl_mapping_entities, Some(audit_log.id)).await?;

        // Update entities
        // # Attention, we are updating in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the update.
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_entities = Vec::new();
        for mut fee_type_gl_mapping in saved {
            fee_type_gl_mapping.gl_code = HeaplessString::try_from("UPDATED").unwrap();
            updated_entities.push(fee_type_gl_mapping);
        }

        let updated = fee_type_gl_mapping_repo.update_batch(updated_entities, Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 3);
        for fee_type_gl_mapping in updated {
            assert_eq!(fee_type_gl_mapping.gl_code.as_str(), "UPDATED");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let fee_type_gl_mapping_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let updated = fee_type_gl_mapping_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}