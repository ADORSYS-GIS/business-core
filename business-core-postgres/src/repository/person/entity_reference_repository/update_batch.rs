use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    person::entity_reference::EntityReferenceModel,
};
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<EntityReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<EntityReferenceModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for EntityReferenceModel")?;
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
                    INSERT INTO entity_reference_audit
                    (id, person_id, entity_role, reference_external_id, reference_details, related_person_id, start_date, end_date, status, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_id)
                .bind(item.entity_role)
                .bind(item.reference_external_id.as_str())
                .bind(item.reference_details)
                .bind(item.related_person_id)
                .bind(item.start_date)
                .bind(item.end_date)
                .bind(item.status)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE entity_reference SET
                    person_id = $2, entity_role = $3, reference_external_id = $4,
                    reference_details = $5, related_person_id = $6, start_date = $7, end_date = $8, status = $9,
                    antecedent_hash = $10, antecedent_audit_log_id = $11, hash = $12, audit_log_id = $13
                    WHERE id = $1 AND hash = $14 AND audit_log_id = $15
                    "#,
                )
                .bind(item.id)
                .bind(item.person_id)
                .bind(item.entity_role)
                .bind(item.reference_external_id.as_str())
                .bind(item.reference_details)
                .bind(item.related_person_id)
                .bind(item.start_date)
                .bind(item.end_date)
                .bind(item.status)
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
                    UPDATE entity_reference_idx SET person_id = $2, reference_external_id_hash = $3 WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.person_id)
                .bind(idx.reference_external_id_hash)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
                    entity_type: AuditEntityType::EntityReference,
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
            let cache = self.entity_reference_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, EntityReferenceModel> for EntityReferenceRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<EntityReferenceModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<EntityReferenceModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::entity_reference_repository::test_utils::create_test_entity_reference;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Charlie Brown");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let mut entity_references = Vec::new();
        for i in 0..3 {
            let entity_reference = create_test_entity_reference(person_id, &format!("UPDATE-{i}"));
            entity_references.push(entity_reference);
        }

        let saved = entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_entity_references = Vec::new();
        for mut entity_reference in saved {
            entity_reference.reference_external_id = HeaplessString::try_from("UPDATED-REF").unwrap();
            updated_entity_references.push(entity_reference);
        }

        let updated = entity_reference_repo.update_batch(updated_entity_references, Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 3);
        for entity_reference in updated {
            assert_eq!(entity_reference.reference_external_id.as_str(), "UPDATED-REF");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let updated = entity_reference_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}