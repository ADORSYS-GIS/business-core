use business_core_db::models::audit::{audit_link::AuditLinkModel, entity_type::EntityType};
use async_trait::async_trait;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::repository::delete_batch::DeleteBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::DocumentRepositoryImpl;

impl DocumentRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &DocumentRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for DocumentModel")?;
        if ids.is_empty() {
            return Ok(0);
        }

        // 1. Load the full entities to be deleted
        let entities_to_delete = repo.load_batch(ids).await?;
        
        let mut deleted_count = 0;
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for entity_opt in entities_to_delete {
            let entity = match entity_opt {
                Some(e) => e,
                None => continue,
            };
            
            // 2. Create a final audit record before deletion
            let mut final_audit_entity = entity.clone();
            final_audit_entity.antecedent_hash = entity.hash;
            final_audit_entity.antecedent_audit_log_id = entity.audit_log_id
                .ok_or("Entity must have audit_log_id for deletion")?;
            final_audit_entity.audit_log_id = Some(audit_log_id);
            final_audit_entity.hash = 0;
            
            let final_hash = hash_as_i64(&final_audit_entity)?;
            final_audit_entity.hash = final_hash;
            
            // 3. Build the audit insert query
            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO person_document_audit
                (id, person_id, document_type, document_path, status, predecessor_1, predecessor_2, predecessor_3, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(final_audit_entity.id)
            .bind(final_audit_entity.person_id)
            .bind(final_audit_entity.document_type.as_str())
            .bind(final_audit_entity.document_path.as_deref())
            .bind(final_audit_entity.status)
            .bind(final_audit_entity.predecessor_1)
            .bind(final_audit_entity.predecessor_2)
            .bind(final_audit_entity.predecessor_3)
            .bind(final_audit_entity.antecedent_hash)
            .bind(final_audit_entity.antecedent_audit_log_id)
            .bind(final_audit_entity.hash)
            .bind(final_audit_entity.audit_log_id);
            
            // 4. Build the entity delete query
            let entity_delete_query = sqlx::query(
                r#"
                DELETE FROM person_document WHERE id = $1
                "#,
            )
            .bind(entity.id);
            
            // 5. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
                entity_type: EntityType::Document,
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
            
            // 6. Execute in transaction (audit first!)
            audit_insert_query.execute(&mut **transaction).await?;
            let result = entity_delete_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;
            
            deleted_count += result.rows_affected() as usize;
        }

        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for DocumentRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::document_repository::test_utils::create_test_document;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;

    fn create_test_audit_log() -> business_core_db::models::audit::audit_log::AuditLogModel {
        business_core_db::models::audit::audit_log::AuditLogModel {
            id: uuid::Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: uuid::Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let document_repo = &ctx.person_repos().document_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let person_id = Uuid::new_v4();
        let mut documents = Vec::new();
        for _ in 0..3 {
            let document = create_test_document(person_id);
            documents.push(document);
        }

        let saved = document_repo.create_batch(documents, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        
        // Create new audit log for delete
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = document_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 3);

        // Verify deletion
        let loaded = document_repo.load_batch(&ids).await?;
        assert!(loaded.iter().all(|item| item.is_none()));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let document_repo = &ctx.person_repos().document_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let person_id = Uuid::new_v4();
        let document = create_test_document(person_id);
        let saved = document_repo.create_batch(vec![document], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        // Create new audit log for delete
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = document_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}