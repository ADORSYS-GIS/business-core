use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
    description::named::NamedModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

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
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for mut entity in items {
            // 1. Save current hash and audit_log_id for antecedent tracking
            let previous_hash = entity.hash;
            let previous_audit_log_id = entity.audit_log_id
                .ok_or("Entity must have audit_log_id for update")?;
            
            // 2. Check if entity has actually changed by recomputing hash
            let mut entity_for_hashing = entity.clone();
            entity_for_hashing.hash = 0;
            
            let computed_hash = hash_as_i64(&entity_for_hashing)?;
            
            // 3. Only proceed with update if entity has changed
            if computed_hash == previous_hash {
                updated_items.push(entity);
                continue;
            }
            
            // 4. Entity has changed, update with new hash and audit_log_id
            entity.antecedent_hash = previous_hash;
            entity.antecedent_audit_log_id = previous_audit_log_id;
            entity.audit_log_id = Some(audit_log_id);
            entity.hash = 0;
            
            let new_computed_hash = hash_as_i64(&entity)?;
            entity.hash = new_computed_hash;
            
            // 5. Build audit insert query
            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO named_audit
                (id, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
            )
            .bind(entity.id)
            .bind(entity.name_l1.as_str())
            .bind(entity.name_l2.as_deref())
            .bind(entity.name_l3.as_deref())
            .bind(entity.name_l4.as_deref())
            .bind(entity.description_l1.as_deref())
            .bind(entity.description_l2.as_deref())
            .bind(entity.description_l3.as_deref())
            .bind(entity.description_l4.as_deref())
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id)
            .bind(entity.hash)
            .bind(entity.audit_log_id);
            
            // 6. Build entity update query
            let rows_affected = sqlx::query(
                r#"
                UPDATE named SET
                    name_l1 = $2,
                    name_l2 = $3,
                    name_l3 = $4,
                    name_l4 = $5,
                    description_l1 = $6,
                    description_l2 = $7,
                    description_l3 = $8,
                    description_l4 = $9,
                    antecedent_hash = $10,
                    antecedent_audit_log_id = $11,
                    hash = $12,
                    audit_log_id = $13
                WHERE id = $1
                  AND hash = $14
                  AND audit_log_id = $15
                "#,
            )
            .bind(entity.id)
            .bind(entity.name_l1.as_str())
            .bind(entity.name_l2.as_deref())
            .bind(entity.name_l3.as_deref())
            .bind(entity.name_l4.as_deref())
            .bind(entity.description_l1.as_deref())
            .bind(entity.description_l2.as_deref())
            .bind(entity.description_l3.as_deref())
            .bind(entity.description_l4.as_deref())
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id)
            .bind(entity.hash)
            .bind(entity.audit_log_id)
            .bind(previous_hash)
            .bind(previous_audit_log_id)
            .execute(&mut **transaction)
            .await?
            .rows_affected();

            if rows_affected == 0 {
                return Err("Concurrent update detected".into());
            }
            
            // 7. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
                entity_type: EntityType::Named,
            };
            let audit_link_query = sqlx::query(
                r#"
                INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(audit_link.audit_log_id)
            .bind(audit_link.entity_id)
            .bind(audit_link.entity_type);
            
            // 8. Execute in transaction (audit first!)
            audit_insert_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;
            
            updated_items.push(entity);
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
    use crate::repository::description::named_repository::test_utils::create_test_named;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;

    fn create_test_audit_log() -> business_core_db::models::audit::audit_log::AuditLogModel {
        business_core_db::models::audit::audit_log::AuditLogModel {
            id: uuid::Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: uuid::Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let named_repo = &ctx.description_repos().named_repository;

        // Create initial audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_named();
        let saved = named_repo.create_batch(vec![entity], Some(audit_log.id)).await?;

        // Update with new audit log
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;

        let mut updated_entity = saved[0].clone();
        updated_entity.name_l1 = HeaplessString::try_from("Updated Name").unwrap();

        let updated = named_repo.update_batch(vec![updated_entity], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].name_l1.as_str(), "Updated Name");
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_no_change() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_named();
        let saved = named_repo.create_batch(vec![entity], Some(audit_log.id)).await?;

        // Update without changing anything
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;

        let unchanged_entity = saved[0].clone();
        let updated = named_repo.update_batch(vec![unchanged_entity], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        // Should still have the original audit_log_id since no change was made
        assert_eq!(updated[0].audit_log_id, Some(audit_log.id));

        Ok(())
    }
}