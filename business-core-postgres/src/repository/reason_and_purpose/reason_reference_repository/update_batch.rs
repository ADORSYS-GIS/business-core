use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
    reason_and_purpose::reason_reference::ReasonReferenceModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonReferenceRepositoryImpl;

impl ReasonReferenceRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<ReasonReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonReferenceModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ReasonReferenceModel")?;
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
                INSERT INTO reason_reference_audit
                (id, reason_id, entity_id, additional_details, entity_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(entity.id)
            .bind(entity.reason_id)
            .bind(entity.entity_id)
            .bind(entity.additional_details.as_deref())
            .bind(entity.entity_type)
            .bind(entity.antecedent_hash)
            .bind(entity.antecedent_audit_log_id)
            .bind(entity.hash)
            .bind(entity.audit_log_id);
            
            // 6. Build entity update query
            let rows_affected = sqlx::query(
                r#"
                UPDATE reason_reference SET
                    reason_id = $2,
                    entity_id = $3,
                    additional_details = $4,
                    entity_type = $5,
                    antecedent_hash = $6,
                    antecedent_audit_log_id = $7,
                    hash = $8,
                    audit_log_id = $9
                WHERE id = $1
                  AND hash = $10
                  AND audit_log_id = $11
                "#,
            )
            .bind(entity.id)
            .bind(entity.reason_id)
            .bind(entity.entity_id)
            .bind(entity.additional_details.as_deref())
            .bind(entity.entity_type)
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
                entity_type: EntityType::ReasonReference,
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
impl UpdateBatch<Postgres, ReasonReferenceModel> for ReasonReferenceRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ReasonReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonReferenceModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::reason_and_purpose::reason_reference_repository::test_utils::create_test_reason_reference;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
    use crate::repository::reason_and_purpose::compliance_metadata_repository::test_utils::test_utils::create_test_compliance_metadata;
    use crate::repository::reason_and_purpose::reason_repository::test_utils::test_utils::create_test_reason_with_compliance_metadata;

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
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        // Create initial audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create compliance metadata and reason first
        let compliance_metadata = create_test_compliance_metadata(Some("GDPR-004"), true, false);
        compliance_metadata_repo.create_batch(vec![compliance_metadata.clone()], Some(audit_log.id)).await?;

        let reason = create_test_reason_with_compliance_metadata("DATA_ANALYSIS", "Data Analysis Reason", Some(compliance_metadata.id));
        let saved_reasons = reason_repo.create_batch(vec![reason], Some(audit_log.id)).await?;
        let reason_id = saved_reasons[0].id;

        let reason_reference = create_test_reason_reference(reason_id, uuid::Uuid::new_v4());
        let saved = reason_reference_repo.create_batch(vec![reason_reference], Some(audit_log.id)).await?;

        // Update with new audit log
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;

        let mut updated_reason_reference = saved[0].clone();
        updated_reason_reference.additional_details = Some(HeaplessString::try_from("Updated details").unwrap());

        let updated = reason_reference_repo.update_batch(vec![updated_reason_reference], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].additional_details.as_ref().map(|s| s.as_str()), Some("Updated details"));
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let updated = reason_reference_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}