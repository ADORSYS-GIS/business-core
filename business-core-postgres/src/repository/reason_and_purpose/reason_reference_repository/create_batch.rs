use async_trait::async_trait;
use business_core_db::models::{
    audit::{audit_link::AuditLinkModel, entity_type::EntityType},
    reason_and_purpose::reason_reference::ReasonReferenceModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonReferenceRepositoryImpl;

impl ReasonReferenceRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &ReasonReferenceRepositoryImpl,
        items: Vec<ReasonReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonReferenceModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ReasonReferenceModel")?;
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

            // 5. Build entity insert query
            let entity_insert_query = sqlx::query(
                r#"
                INSERT INTO reason_reference
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

            // 6. Create audit link to track the entity modification in the transaction
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
impl CreateBatch<Postgres, ReasonReferenceModel> for ReasonReferenceRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<ReasonReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonReferenceModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::reason_and_purpose::reason_reference_repository::test_utils::create_test_reason_reference;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use crate::repository::reason_and_purpose::compliance_metadata_repository::test_utils::test_utils::create_test_compliance_metadata;
    use crate::repository::reason_and_purpose::reason_repository::test_utils::test_utils::{create_test_reason_with_compliance_metadata};

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
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create compliance metadata and reason first
        let compliance_metadata = create_test_compliance_metadata(Some("GDPR-001"), true, false);
        compliance_metadata_repo.create_batch(vec![compliance_metadata.clone()], Some(audit_log.id)).await?;

        let reason = create_test_reason_with_compliance_metadata("DATA_PROCESSING", "Data Processing Reason", Some(compliance_metadata.id));
        let saved_reasons = reason_repo.create_batch(vec![reason], Some(audit_log.id)).await?;
        let reason_id = saved_reasons[0].id;

        let mut reason_references = Vec::new();
        for _i in 0..3 {
            let reason_reference = create_test_reason_reference(reason_id, uuid::Uuid::new_v4());
            reason_references.push(reason_reference);
        }

        let saved = reason_reference_repo
            .create_batch(reason_references, Some(audit_log.id))
            .await?;

        assert_eq!(saved.len(), 3);
        for entity in &saved {
            assert!(entity.hash != 0, "Hash should be computed");
            assert_eq!(entity.audit_log_id, Some(audit_log.id));
            assert_eq!(entity.reason_id, reason_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        let audit_log = create_test_audit_log();
        let saved = reason_reference_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved.len(), 0);

        Ok(())
    }
}