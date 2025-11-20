use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
};
use business_core_db::repository::delete_batch::DeleteBatch;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ActivityLogRepositoryImpl;

impl ActivityLogRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &ActivityLogRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ActivityLogModel")?;
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
                INSERT INTO person_activity_log_audit
                (id, person_id, activity_summary, predecessor_1, predecessor_2, predecessor_3, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(final_audit_entity.id)
            .bind(final_audit_entity.person_id)
            .bind(final_audit_entity.activity_summary.as_deref())
            .bind(final_audit_entity.predecessor_1)
            .bind(final_audit_entity.predecessor_2)
            .bind(final_audit_entity.predecessor_3)
            .bind(final_audit_entity.hash)
            .bind(final_audit_entity.audit_log_id)
            .bind(final_audit_entity.antecedent_hash)
            .bind(final_audit_entity.antecedent_audit_log_id);
            
            // 4. Build the entity delete query
            let entity_delete_query = sqlx::query(
                r#"
                DELETE FROM person_activity_log WHERE id = $1
                "#,
            )
            .bind(entity.id);
            
            // 5. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
                entity_type: EntityType::ActivityLog,
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
            entity_delete_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;
            
            deleted_count += 1;
        }

        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for ActivityLogRepositoryImpl {
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
    use crate::repository::person::activity_log_repository::test_utils::create_test_activity_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use crate::repository::person::person_repository::test_utils::create_test_person;
    use business_core_db::models::person::person::PersonType;

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
        let person_repo = &ctx.person_repos().person_repository;
        let activity_log_repo = &ctx.person_repos().activity_log_repository;

        // Create entities
        let create_audit_log = create_test_audit_log();
        audit_log_repo.create(&create_audit_log).await?;

        // Create a person first
        let person = create_test_person("Test Person", PersonType::Natural);
        let saved_persons = person_repo.create_batch(vec![person], Some(create_audit_log.id)).await?;
        let person_id = saved_persons[0].id;
        
        let activity_logs = vec![create_test_activity_log(person_id)];
        let saved = activity_log_repo.create_batch(activity_logs, Some(create_audit_log.id)).await?;

        let ids: Vec<uuid::Uuid> = saved.iter().map(|e| e.id).collect();

        // Delete entities
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = activity_log_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1);

        // Verify deletion
        let loaded = activity_log_repo.load_batch(&ids).await?;
        assert!(loaded[0].is_none());

        Ok(())
    }
}