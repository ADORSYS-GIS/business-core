use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    description::named::NamedModel,
};
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::NamedRepositoryImpl;

impl NamedRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for NamedModel")?;
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
                    INSERT INTO named_audit
                    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                    "#,
                )
                .bind(item.id)
                .bind(item.entity_type)
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_deref())
                .bind(item.name_l3.as_deref())
                .bind(item.name_l4.as_deref())
                .bind(item.description_l1.as_deref())
                .bind(item.description_l2.as_deref())
                .bind(item.description_l3.as_deref())
                .bind(item.description_l4.as_deref())
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE named SET
                    entity_type = $2,
                    name_l1 = $3,
                    name_l2 = $4,
                    name_l3 = $5,
                    name_l4 = $6,
                    description_l1 = $7,
                    description_l2 = $8,
                    description_l3 = $9,
                    description_l4 = $10,
                    antecedent_hash = $11,
                    antecedent_audit_log_id = $12,
                    hash = $13,
                    audit_log_id = $14
                    WHERE id = $1 AND hash = $15 AND audit_log_id = $16
                    "#,
                )
                .bind(item.id)
                .bind(item.entity_type)
                .bind(item.name_l1.as_str())
                .bind(item.name_l2.as_deref())
                .bind(item.name_l3.as_deref())
                .bind(item.name_l4.as_deref())
                .bind(item.description_l1.as_deref())
                .bind(item.description_l2.as_deref())
                .bind(item.description_l3.as_deref())
                .bind(item.description_l4.as_deref())
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
                    UPDATE named_idx SET
                    entity_type = $2
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.entity_type)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::Named,
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
            let cache = self.named_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, NamedModel> for NamedRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
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
    use crate::repository::description::named_repository::test_utils::create_test_named;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut named_entities = Vec::new();
        for i in 0..3 {
            let named = create_test_named(&format!("Original Entity {i}"));
            named_entities.push(named);
        }

        let saved = named_repo.create_batch(named_entities, Some(audit_log.id)).await?;

        // Update entities
        // # Attention, we are updating in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the update.
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_entities = Vec::new();
        for mut named in saved {
            named.name_l1 = HeaplessString::try_from("Updated Entity").unwrap();
            updated_entities.push(named);
        }

        let updated = named_repo.update_batch(updated_entities, Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 3);
        for named in updated {
            assert_eq!(named.name_l1.as_str(), "Updated Entity");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let named_repo = &ctx.description_repos().named_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let updated = named_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}