use business_core_db::models::audit::{AuditLinkModel, EntityType};
use async_trait::async_trait;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &EntityReferenceRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for EntityReferenceModel")?;
        if ids.is_empty() {
            return Ok(0);
        }

        let entities_to_delete = repo.load_batch(ids).await?;
        let mut deleted_count = 0;

        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

            for entity in entities_to_delete.into_iter().flatten() {
                let mut final_audit_entity = entity.clone();
                final_audit_entity.antecedent_hash = entity.hash;
                final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;
                final_audit_entity.audit_log_id = Some(audit_log_id);
                final_audit_entity.hash = 0;

                let final_hash = hash_as_i64(&final_audit_entity)?;
                final_audit_entity.hash = final_hash;

                sqlx::query(
                    r#"
                    INSERT INTO entity_reference_audit
                    (id, person_id, entity_role, reference_external_id, reference_details_l1, reference_details_l2, reference_details_l3, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    "#,
                )
                .bind(final_audit_entity.id)
                .bind(final_audit_entity.person_id)
                .bind(final_audit_entity.entity_role)
                .bind(final_audit_entity.reference_external_id.as_str())
                .bind(final_audit_entity.reference_details_l1.as_deref())
                .bind(final_audit_entity.reference_details_l2.as_deref())
                .bind(final_audit_entity.reference_details_l3.as_deref())
                .bind(final_audit_entity.antecedent_hash)
                .bind(final_audit_entity.antecedent_audit_log_id)
                .bind(final_audit_entity.hash)
                .bind(final_audit_entity.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let result = sqlx::query(r#"DELETE FROM entity_reference WHERE id = $1"#)
                    .bind(entity.id)
                    .execute(&mut **transaction)
                    .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: entity.id,
                    entity_type: EntityType::EntityReference,
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
                
                deleted_count += result.rows_affected() as usize;
            }
        }
        
        {
            let cache = repo.entity_reference_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for EntityReferenceRepositoryImpl {
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
    use crate::repository::person::entity_reference_repository::test_utils::create_test_entity_reference;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("David Miller");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let mut entity_references = Vec::new();
        for i in 0..3 {
            let entity_reference = create_test_entity_reference(person_id, &format!("DELETE-{}", i));
            entity_references.push(entity_reference);
        }

        let saved = entity_reference_repo.create_batch(entity_references, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = entity_reference_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Eve Wilson");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let entity_reference = create_test_entity_reference(person_id, "DELETE-TEST");

        let saved = entity_reference_repo.create_batch(vec![entity_reference], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = entity_reference_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}