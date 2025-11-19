use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
    person::activity_log::ActivityLogModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ActivityLogRepositoryImpl;

impl ActivityLogRepositoryImpl {
    pub(super) async fn update_batch_impl(
        repo: &ActivityLogRepositoryImpl,
        items: Vec<ActivityLogModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ActivityLogModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ActivityLogModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut tx = repo.executor.tx.lock().await;
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
                INSERT INTO person_activity_log_audit
                (id, person_id, activity_summary, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.activity_summary.as_deref())
            .bind(entity.hash)
            .bind(entity.audit_log_id)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id);
            
            // 6. Build entity update query
            let entity_update_query = sqlx::query(
                r#"
                UPDATE person_activity_log SET
                    person_id = $2,
                    activity_summary = $3,
                    hash = $4,
                    audit_log_id = $5,
                    antecedent_hash = $6,
                    antecedent_audit_log_id = $7
                WHERE id = $1
                  AND hash = $6
                  AND audit_log_id = $7
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.activity_summary.as_deref())
            .bind(entity.hash)
            .bind(entity.audit_log_id)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id);
            
            // 7. Create audit link
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
            
            // 8. Execute in transaction (audit first!)
            audit_insert_query.execute(&mut **transaction).await?;
            entity_update_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;
            
            updated_items.push(entity);
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, ActivityLogModel> for ActivityLogRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ActivityLogModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ActivityLogModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::activity_log_repository::test_utils::create_test_activity_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
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
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let activity_log_repo = &ctx.person_repos().activity_log_repository;

        // Create initial entities
        let create_audit_log = create_test_audit_log();
        audit_log_repo.create(&create_audit_log).await?;

        // Create a person first
        let person = create_test_person("Test Person", PersonType::Natural);
        let saved_persons = person_repo.create_batch(vec![person], Some(create_audit_log.id)).await?;
        let person_id = saved_persons[0].id;
        
        let activity_logs = vec![create_test_activity_log(person_id)];
        let saved = activity_log_repo.create_batch(activity_logs, Some(create_audit_log.id)).await?;

        // Update entities
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        
        let mut updated_activity_logs = Vec::new();
        for mut entity in saved {
            entity.activity_summary = Some(HeaplessString::try_from("Updated activity summary").unwrap());
            updated_activity_logs.push(entity);
        }

        let updated = activity_log_repo.update_batch(updated_activity_logs, Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].activity_summary.as_ref().unwrap().as_str(), "Updated activity summary");
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }
}