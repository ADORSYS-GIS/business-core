use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
    person::compliance_status::ComplianceStatusModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceStatusRepositoryImpl;

impl ComplianceStatusRepositoryImpl {
    pub(super) async fn update_batch_impl(
        repo: &ComplianceStatusRepositoryImpl,
        items: Vec<ComplianceStatusModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ComplianceStatusModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ComplianceStatusModel")?;
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
                INSERT INTO person_compliance_status_audit
                (id, person_id, kyc_status, sanctions_checked, last_screening_date, predecessor_1, predecessor_2, predecessor_3, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.kyc_status)
            .bind(entity.sanctions_checked)
            .bind(entity.last_screening_date)
            .bind(entity.predecessor_1)
            .bind(entity.predecessor_2)
            .bind(entity.predecessor_3)
            .bind(entity.hash)
            .bind(entity.audit_log_id)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id);
            
            // 6. Build entity update query
            let entity_update_query = sqlx::query(
                r#"
                UPDATE person_compliance_status SET
                    person_id = $2,
                    kyc_status = $3,
                    sanctions_checked = $4,
                    last_screening_date = $5,
                    predecessor_1 = $6,
                    predecessor_2 = $7,
                    predecessor_3 = $8,
                    hash = $9,
                    audit_log_id = $10,
                    antecedent_hash = $11,
                    antecedent_audit_log_id = $12
                WHERE id = $1
                  AND hash = $11
                  AND audit_log_id = $12
                "#,
            )
            .bind(entity.id)
            .bind(entity.person_id)
            .bind(entity.kyc_status)
            .bind(entity.sanctions_checked)
            .bind(entity.last_screening_date)
            .bind(entity.predecessor_1)
            .bind(entity.predecessor_2)
            .bind(entity.predecessor_3)
            .bind(entity.hash)
            .bind(entity.audit_log_id)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id);
            
            // 7. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: entity.id,
                entity_type: EntityType::ComplianceStatus,
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
impl UpdateBatch<Postgres, ComplianceStatusModel> for ComplianceStatusRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ComplianceStatusModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ComplianceStatusModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::compliance_status_repository::test_utils::create_test_compliance_status;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use business_core_db::models::person::compliance_status::KycStatus;

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
        let compliance_status_repo = &ctx.person_repos().compliance_status_repository;

        // Create initial entities
        let create_audit_log = create_test_audit_log();
        audit_log_repo.create(&create_audit_log).await?;
        
        let entity = create_test_compliance_status(uuid::Uuid::new_v4());
        let saved = compliance_status_repo.create_batch(vec![entity], Some(create_audit_log.id)).await?;

        // Update entities
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        
        let mut updated_entity = saved[0].clone();
        updated_entity.kyc_status = KycStatus::Approved;
        updated_entity.sanctions_checked = true;

        let updated = compliance_status_repo.update_batch(vec![updated_entity], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].kyc_status, KycStatus::Approved);
        assert_eq!(updated[0].sanctions_checked, true);
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }
}