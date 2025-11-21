use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, audit_entity_type::AuditEntityType},
    description::named::NamedModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::NamedRepositoryImpl;

impl NamedRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &NamedRepositoryImpl,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for NamedModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for mut entity in items {
            // 1. Create a copy of entity for hashing
            let mut entity_for_hashing = entity.clone();
            entity_for_hashing.hash = 0;  // Must be 0 before hashing
            entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing

            // 2. Compute hash
            let computed_hash = hash_as_i64(&entity_for_hashing)?;

            // 3. Update original entity with computed hash and new audit_log_id
            entity.hash = computed_hash;
            entity.audit_log_id = Some(audit_log_id);

            // 4. Build audit insert query - inserts the entity
            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO named_audit
                (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                "#,
            )
            .bind(entity.id)
            .bind(entity.entity_type)
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

            // 5. Build entity insert query
            let entity_insert_query = sqlx::query(
                r#"
                INSERT INTO named
                (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                "#,
            )
            .bind(entity.id)
            .bind(entity.entity_type)
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

            // 6. Create audit link to track the entity modification in the transaction
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
                entity_type: AuditEntityType::Named,
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

            // 7. Execute in transaction (audit first!)
            audit_insert_query.execute(&mut **transaction).await?;
            entity_insert_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;

            saved_items.push(entity);
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, NamedModel> for NamedRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<NamedModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<NamedModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::description::named_repository::test_utils::create_test_named;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;

    fn create_test_audit_log() -> business_core_db::models::audit::audit_log::AuditLogModel {
        business_core_db::models::audit::audit_log::AuditLogModel {
            id: uuid::Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: uuid::Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut entities = Vec::new();
        for _i in 0..3 {
            entities.push(create_test_named());
        }

        let saved = named_repo
            .create_batch(entities, Some(audit_log.id))
            .await?;

        assert_eq!(saved.len(), 3);
        for entity in &saved {
            assert!(entity.hash != 0, "Hash should be computed");
            assert_eq!(entity.audit_log_id, Some(audit_log.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        let saved = named_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved.len(), 0);

        Ok(())
    }
}