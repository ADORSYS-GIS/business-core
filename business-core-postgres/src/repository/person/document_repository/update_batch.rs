use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, audit_entity_type::AuditEntityType},
    person::document::DocumentModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::DocumentRepositoryImpl;

impl DocumentRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<DocumentModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<DocumentModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for DocumentModel")?;
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
                INSERT INTO person_document_audit
                (id, person_id, document_type, document_path, status, predecessor_1, predecessor_2, predecessor_3, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.document_type.as_str())
            .bind(entity.document_path.as_deref())
            .bind(entity.status)
            .bind(entity.predecessor_1)
            .bind(entity.predecessor_2)
            .bind(entity.predecessor_3)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id)
            .bind(entity.hash)
            .bind(entity.audit_log_id);
            
            // 6. Build entity update query
            let rows_affected = sqlx::query(
                r#"
                UPDATE person_document SET
                    person_id = $2,
                    document_type = $3,
                    document_path = $4,
                    status = $5,
                    predecessor_1 = $6,
                    predecessor_2 = $7,
                    predecessor_3 = $8,
                    antecedent_hash = $9,
                    antecedent_audit_log_id = $10,
                    hash = $11,
                    audit_log_id = $12
                WHERE id = $1
                  AND hash = $13
                  AND audit_log_id = $14
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.document_type.as_str())
            .bind(entity.document_path.as_deref())
            .bind(entity.status)
            .bind(entity.predecessor_1)
            .bind(entity.predecessor_2)
            .bind(entity.predecessor_3)
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
                entity_type: AuditEntityType::Document,
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
impl UpdateBatch<Postgres, DocumentModel> for DocumentRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<DocumentModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<DocumentModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::document_repository::test_utils::create_test_document;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use business_core_db::models::person::document::DocumentStatus;

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
        let document_repo = &ctx.person_repos().document_repository;

        // Create initial audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let person_id = uuid::Uuid::new_v4();
        let document = create_test_document(person_id);
        let saved = document_repo.create_batch(vec![document], Some(audit_log.id)).await?;

        // Update with new audit log
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;

        let mut updated_document = saved[0].clone();
        updated_document.status = DocumentStatus::Verified;

        let updated = document_repo.update_batch(vec![updated_document], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].status, DocumentStatus::Verified);
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let document_repo = &ctx.person_repos().document_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let updated = document_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}